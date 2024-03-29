// Copyright 2023 MeshX Contributors. All rights reserved.
// Copyright 2018 The Fuchsia Authors. All rights reserved.
// Use of this source code is governed by a BSD-style license that can be
// found in the LICENSE file.

//! An implementation of a client for a fidl interface.
use {
    crate::{
        encoding::{
            decode_transaction_header, maybe_overflowing_after_encode, maybe_overflowing_decode, Decode, Decoder,
            DynamicFlags, Encode, Encoder, EpitaphBody, TransactionHeader, TransactionMessage, TransactionMessageType,
            TypeMarker,
        },
        handle::{AsyncChannel, HandleDisposition, MessageBufEtc},
        Error,
    },
    fiber_status as fx_status,
    futures::{
        future::{self, FusedFuture, Future, FutureExt, Map, MaybeDone},
        ready,
        stream::{FusedStream, Stream},
        task::{noop_waker, ArcWake, Context, Poll, Waker},
    },
    parking_lot::Mutex,
    slab::Slab,
    std::{collections::VecDeque, marker::Unpin, mem, pin::Pin, sync::Arc},
};

/// Decodes the body of `buf` as the FIDL type `T`.
#[doc(hidden)] // only exported for use in macros or generated code
pub fn decode_transaction_body<T: TypeMarker, const OVERFLOWABLE: bool>(
    mut buf: MessageBufEtc,
) -> Result<T::Owned, Error> {
    let (bytes, handles) = buf.split_mut();
    let (header, body_bytes) = decode_transaction_header(bytes)?;
    let mut output = Decode::<T>::new_empty();

    if OVERFLOWABLE {
        maybe_overflowing_decode::<T>(&header, body_bytes, handles, &mut output)?;
    } else {
        Decoder::decode_into::<T>(&header, body_bytes, handles, &mut output)?;
    }

    Ok(output)
}

/// A MIDL client which can be used to send buffers and receive responses via a channel.
#[derive(Debug, Clone)]
pub struct Client {
    inner: Arc<ClientInner>,
}

/// A future representing the decoded and transformed response to a FIDL query.
pub type DecodedQueryResponseFut<T> = Map<MessageResponse, fn(Result<MessageBufEtc, Error>) -> Result<T, Error>>;

/// A future representing the result of a MIDL query, with early error detection available if the
/// message couldn't be sent.
#[derive(Debug)]
#[must_use = "futures do nothing unless you `.await` or poll them"]
pub struct QueryResponseFut<T>(pub MaybeDone<DecodedQueryResponseFut<T>>);

impl<T> QueryResponseFut<T> {
    /// Check to see if the query has an error. If there was en error sending, this returns it and
    /// the error is returned, otherwise it returns self, which can then be awaited on:
    /// i.e. match echo_proxy.echo("something").check() {
    ///      Err(e) => error!("Couldn't send: {}", e),
    ///      Ok(fut) => fut.await
    /// }
    pub fn check(self) -> Result<Self, Error> {
        match self.0 {
            MaybeDone::Done(Err(e)) => Err(e),
            x => Ok(QueryResponseFut(x)),
        }
    }
}

impl<T: Unpin> FusedFuture for QueryResponseFut<T> {
    fn is_terminated(&self) -> bool {
        match self.0 {
            MaybeDone::Gone => true,
            _ => false,
        }
    }
}

impl<T: Unpin> Future for QueryResponseFut<T> {
    type Output = Result<T, Error>;
    fn poll(mut self: Pin<&mut Self>, cx: &mut Context<'_>) -> Poll<Self::Output> {
        ready!(self.0.poll_unpin(cx));
        let maybe_done = Pin::new(&mut self.0);
        Poll::Ready(maybe_done.take_output().unwrap_or(Err(Error::PollAfterCompletion)))
    }
}

/// A message interest id.
#[derive(Debug, Copy, Clone, PartialEq, Eq)]
struct InterestId(usize);

impl InterestId {
    fn from_txid(txid: Txid) -> Self {
        InterestId(txid.0 as usize - 1)
    }
    fn as_raw_id(&self) -> usize {
        self.0
    }
}

/// A MIDL transaction id. Will not be zero for a message that includes a response.
#[derive(Debug, Copy, Clone, PartialEq, Eq)]
pub struct Txid(u32);

impl Txid {
    fn from_interest_id(int_id: InterestId) -> Self {
        Txid((int_id.0 + 1) as u32)
    }
    /// Get the raw u32 transaction ID.
    pub fn as_raw_id(&self) -> u32 {
        self.0
    }
}

impl From<u32> for Txid {
    fn from(txid: u32) -> Self {
        Self(txid)
    }
}

impl Client {
    /// Create a new client.
    ///
    /// `channel` is the asynchronous channel over which data is sent and received.
    /// `event_ordinals` are the ordinals on which events will be received.
    pub fn new(channel: AsyncChannel, protocol_name: &'static str) -> Client {
        Client {
            inner: Arc::new(ClientInner {
                channel,
                message_interests: Mutex::new(Slab::<MessageInterest>::new()),
                event_channel: Mutex::default(),
                epitaph: Mutex::default(),
                protocol_name,
            }),
        }
    }

    /// Get a reference to the client's underlying channel.
    pub fn as_channel(&self) -> &AsyncChannel {
        &self.inner.channel
    }

    /// Attempt to convert the `Client` back into a channel.
    ///
    /// This will only succeed if there are no active clones of this `Client`,
    /// no currently-alive `EventReceiver` or `MessageResponse`s that came from
    /// this `Client`, and no outstanding messages awaiting a response, even if
    /// that response will be discarded.
    pub fn into_channel(self) -> Result<AsyncChannel, Self> {
        // We need to check the message_interests table to make sure there are
        // no outstanding interests, since an interest might still exist even if
        // all EventReceivers and MessageResponses have been dropped. That would
        // lead to returning an AsyncChannel which could then later receive the
        // outstanding response unexpectedly.
        //
        // We do try_unwrap before checking the message_interests to avoid a
        // race where another thread inserts a new value into message_interests
        // after we check message_interests.is_empty(), but before we get to
        // try_unwrap. This forces us to create a new Arc if message_interests
        // isn't empty, since try_unwrap destroys the original Arc.
        match Arc::try_unwrap(self.inner) {
            Ok(inner) => {
                if inner.message_interests.lock().is_empty() || inner.channel.is_closed() {
                    Ok(inner.channel)
                } else {
                    // This creates a new arc if there are outstanding
                    // interests. This is ok because we never create any weak
                    // references to ClientInner, otherwise doing this would
                    // detach weak references.
                    Err(Self { inner: Arc::new(inner) })
                }
            }
            Err(inner) => Err(Self { inner }),
        }
    }

    /// Retrieve the stream of event messages for the `Client`.
    /// Panics if the stream was already taken.
    pub fn take_event_receiver(&self) -> EventReceiver {
        {
            let mut lock = self.inner.event_channel.lock();
            if let EventListener::None = lock.listener {
                lock.listener = EventListener::WillPoll;
            } else {
                panic!("Event stream was already taken");
            }
        }
        EventReceiver {
            inner: self.inner.clone(),
            state: EventReceiverState::Active,
        }
    }

    /// Encodes and sends a request without expecting a response.
    pub fn send<T: TypeMarker, const OVERFLOWABLE: bool>(
        &self,
        body: impl Encode<T>,
        ordinal: u64,
        dynamic_flags: DynamicFlags,
    ) -> Result<(), Error> {
        let msg = TransactionMessage {
            header: TransactionHeader::new(0, ordinal, dynamic_flags),
            body,
        };
        crate::encoding::with_tls_encoded::<TransactionMessageType<T>, (), OVERFLOWABLE>(msg, |bytes, handles| {
            self.send_raw(bytes, handles)
        })
    }

    /// Encodes and sends a request. Returns a future that decodes the response.
    pub fn send_query<
        Request: TypeMarker,
        Response: TypeMarker,
        const REQUEST_ENCODE_OVERFLOWABLE: bool,
        const RESPONSE_DECODE_OVERFLOWABLE: bool,
    >(
        &self,
        body: impl Encode<Request>,
        ordinal: u64,
        dynamic_flags: DynamicFlags,
    ) -> QueryResponseFut<Response::Owned> {
        self.send_query_and_decode::<Request, Response::Owned, REQUEST_ENCODE_OVERFLOWABLE>(
            body,
            ordinal,
            dynamic_flags,
            |buf| buf.and_then(decode_transaction_body::<Response, RESPONSE_DECODE_OVERFLOWABLE>),
        )
    }

    /// Encodes and sends a request. Returns a future that decodes the response
    /// using the given `decode` function.
    pub fn send_query_and_decode<Request: TypeMarker, Output, const REQUEST_ENCODE_OVERFLOWABLE: bool>(
        &self,
        body: impl Encode<Request>,
        ordinal: u64,
        dynamic_flags: DynamicFlags,
        decode: fn(Result<MessageBufEtc, Error>) -> Result<Output, Error>,
    ) -> QueryResponseFut<Output> {
        let send_result = self.send_raw_query(|tx_id, bytes, handles| {
            let msg = TransactionMessage {
                header: TransactionHeader::new(tx_id.as_raw_id(), ordinal, dynamic_flags),
                body,
            };
            Encoder::encode::<TransactionMessageType<Request>>(bytes, handles, msg)?;
            if REQUEST_ENCODE_OVERFLOWABLE {
                maybe_overflowing_after_encode(bytes, handles)?;
            }
            Ok(())
        });
        QueryResponseFut(match send_result {
            Ok(res_fut) => future::maybe_done(res_fut.map(decode)),
            Err(e) => MaybeDone::Done(Err(e)),
        })
    }

    /// Sends a raw message without expecting a response.
    pub fn send_raw(&self, bytes: &[u8], handles: &mut Vec<HandleDisposition<'_>>) -> Result<(), Error> {
        match self.inner.channel.write_etc(bytes, handles) {
            Ok(()) | Err(fx_status::Status::PEER_CLOSED) => Ok(()),
            Err(e) => Err(Error::ClientWrite(e)),
        }
    }

    /// Sends a raw query and receives a response future.
    pub fn send_raw_query<F>(&self, encode_msg: F) -> Result<MessageResponse, Error>
    where
        F: for<'a, 'b> FnOnce(Txid, &'a mut Vec<u8>, &'b mut Vec<HandleDisposition<'static>>) -> Result<(), Error>,
    {
        let id = self.inner.register_msg_interest();

        crate::encoding::with_tls_encode_buf(|bytes, handles| {
            encode_msg(Txid::from_interest_id(id), bytes, handles)?;
            self.send_raw(bytes, handles)
        })?;

        Ok(MessageResponse {
            id: Txid::from_interest_id(id),
            client: Some(self.inner.clone()),
        })
    }
}

#[must_use]
/// A future which polls for the response to a client message.
#[derive(Debug)]
pub struct MessageResponse {
    id: Txid,
    // `None` if the message response has been received
    client: Option<Arc<ClientInner>>,
}

impl Unpin for MessageResponse {}

impl Future for MessageResponse {
    type Output = Result<MessageBufEtc, Error>;
    fn poll(mut self: Pin<&mut Self>, cx: &mut Context<'_>) -> Poll<Self::Output> {
        let this = &mut *self;
        let res;
        {
            let client = this.client.as_ref().ok_or(Error::PollAfterCompletion)?;
            res = client.poll_recv_msg_response(this.id, cx);
        }
        // Drop the client reference if the response has been received
        if let Poll::Ready(Ok(_)) = res {
            this.client.take().expect("MessageResponse polled after completion");
        }
        res
    }
}

impl Drop for MessageResponse {
    fn drop(&mut self) {
        if let Some(client) = &self.client {
            client.deregister_msg_interest(InterestId::from_txid(self.id));
        }
    }
}

/// An enum reprenting either a resolved message interest or a task on which to alert
/// that a response message has arrived.
#[derive(Debug)]
enum MessageInterest {
    /// A new `MessageInterest`
    WillPoll,
    /// A task is waiting to receive a response, and can be awoken with `Waker`.
    Waiting(Waker),
    /// A message has been received, and a task will poll to receive it.
    Received(MessageBufEtc),
    /// A message has not been received, but the person interested in the response
    /// no longer cares about it, so the message should be discared upon arrival.
    Discard,
}

impl MessageInterest {
    /// Check if a message has been received.
    fn is_received(&self) -> bool {
        if let MessageInterest::Received(_) = *self {
            true
        } else {
            false
        }
    }

    fn unwrap_received(self) -> MessageBufEtc {
        if let MessageInterest::Received(buf) = self {
            buf
        } else {
            panic!("EXPECTED received message")
        }
    }

    /// Registers the waker from `cx` if the message has not already been received, replacing any
    /// previous waker registered.
    fn register(&mut self, cx: &mut Context<'_>) {
        if self.is_received() {
            return;
        }
        if let Self::Discard = self {
            panic!("Polled a discarded MessageReceiver?!");
        }
        // Must be either WillPoll or Waiting, replace the waker.
        *self = Self::Waiting(cx.waker().clone());
    }

    /// Receive a message for this MessageInterest, waking the waiter if they are waiting to
    /// poll.
    /// Returns true if the task interested in the response no longer cares about it, in which
    /// case this can be cleaned up.
    fn receive(&mut self, message: MessageBufEtc) -> bool {
        if let Self::Discard = self {
            return true;
        } else if let Self::Waiting(waker) = mem::replace(self, Self::Received(message)) {
            waker.wake();
        }
        false
    }

    /// Wake the interested task, if it is waiting, putting it in a WillPoll state.
    /// This function is idempotent.
    fn wake(&mut self) {
        if let Self::Waiting(waker) = self {
            waker.wake_by_ref();
            *self = Self::WillPoll;
        }
    }
}

#[derive(Debug)]
enum EventReceiverState {
    Active,
    Epitaph,
    Terminated,
}

/// A stream of events as `MessageBufEtc`s.
#[derive(Debug)]
pub struct EventReceiver {
    inner: Arc<ClientInner>,
    state: EventReceiverState,
}

impl Unpin for EventReceiver {}
impl FusedStream for EventReceiver {
    fn is_terminated(&self) -> bool {
        match self.state {
            EventReceiverState::Terminated => true,
            _ => false,
        }
    }
}

/// This implementation holds up two invariants
///   (1) After `None` is returned, the next poll panics
///   (2) Until this instance is dropped, no other EventReceiver may claim the
///       event channel by calling Client::take_event_receiver.
impl Stream for EventReceiver {
    type Item = Result<MessageBufEtc, Error>;
    fn poll_next(mut self: Pin<&mut Self>, cx: &mut Context<'_>) -> Poll<Option<Self::Item>> {
        match self.state {
            EventReceiverState::Active => {}
            EventReceiverState::Terminated => {
                panic!("polled EventReceiver after `None`");
            }
            EventReceiverState::Epitaph => {
                self.state = EventReceiverState::Terminated;
                return Poll::Ready(None);
            }
        }
        Poll::Ready(match ready!(self.inner.as_ref().poll_recv_event(cx)) {
            Ok(x) => Some(Ok(x)),
            Err(Error::ClientChannelClosed {
                status: fx_status::Status::PEER_CLOSED,
                ..
            }) => {
                // The channel is closed, with no epitaph. Set our internal state so that on
                // the next poll_next() we panic and is_terminated() returns an appropriate value.
                self.state = EventReceiverState::Terminated;
                None
            }
            err @ Err(Error::ClientChannelClosed { .. }) => {
                // The channel is closed with an epitaph. Return the epitaph and set our internal
                // state so that on the next poll_next() we return a None and terminate the stream.
                self.state = EventReceiverState::Epitaph;
                Some(err)
            }
            Err(e) => Some(Err(e)),
        })
    }
}

impl Drop for EventReceiver {
    fn drop(&mut self) {
        self.inner.event_channel.lock().listener = EventListener::None;
    }
}

#[derive(Debug, Default)]
struct EventChannel {
    listener: EventListener,
    queue: VecDeque<MessageBufEtc>,
}

#[derive(Debug)]
enum EventListener {
    /// No one is listening for the event
    None,
    /// Someone is listening for the event but has not yet polled
    WillPoll,
    /// Someone is listening for the event and can be woken via the `Waker`
    Some(Waker),
}

impl Default for EventListener {
    fn default() -> Self {
        EventListener::None
    }
}

impl EventListener {
    /// Wakes this, putting it in a WillPoll State until it is polled again.
    fn wake(&mut self) {
        *self = match mem::replace(self, Self::None) {
            Self::Some(waker) => {
                waker.wake();
                Self::WillPoll
            }
            x => x,
        };
    }
}

struct CombinedWaker {
    wakers: Vec<Waker>,
}

impl CombinedWaker {
    fn make_waker(wakers: Vec<Waker>) -> Waker {
        futures::task::waker(Arc::new(Self { wakers }))
    }
}

impl ArcWake for CombinedWaker {
    fn wake_by_ref(arc_self: &Arc<Self>) {
        for waker in &arc_self.wakers {
            waker.wake_by_ref();
        }
    }
}

/// A shared client channel which tracks EXPECTED and received responses
#[derive(Debug)]
struct ClientInner {
    /// The channel that leads to the server we are connected to.
    channel: AsyncChannel,
    /// A map of MessageInterests, which track state of responses to two-way
    /// messages.
    /// An interest is registered here with `register_msg_interest` and deregistered
    /// by either retrieving a message via a call to `poll_recv_msg_response` or manually
    /// deregistering with `deregister_msg_interest`
    message_interests: Mutex<Slab<MessageInterest>>,
    /// A queue of received events and a waker for the task to receive them.
    event_channel: Mutex<EventChannel>,
    /// The server provided epitaph, or None if the channel is not closed.
    epitaph: Mutex<Option<fx_status::Status>>,
    /// The `ProtocolMarker::DEBUG_NAME` for the service this client connects to.
    protocol_name: &'static str,
}

impl ClientInner {
    /// Registers interest in a response message.
    ///
    /// This function returns a `usize` ID which should be used to send a message
    /// via the channel. Responses are then received using `poll_recv`.
    fn register_msg_interest(&self) -> InterestId {
        // TODO(cramertj) use `try_from` here and assert that the conversion from
        // `usize` to `u32` hasn't overflowed.
        InterestId(self.message_interests.lock().insert(MessageInterest::WillPoll))
    }

    fn poll_recv_event(&self, cx: &mut Context<'_>) -> Poll<Result<MessageBufEtc, Error>> {
        {
            // Update the EventListener with the latest waker, remove any stale WillPoll state
            let mut lock = self.event_channel.lock();
            lock.listener = EventListener::Some(cx.waker().clone());
        }
        // Process any data on the channel, registering any tasks still waiting to wake when the
        // channel becomes ready.
        let epitaph = self.recv_all()?;
        let mut lock = self.event_channel.lock();
        if let Some(msg_buf) = lock.queue.pop_front() {
            Poll::Ready(Ok(msg_buf))
        } else {
            if let Some(status) = epitaph {
                Poll::Ready(Err(Error::ClientChannelClosed {
                    status,
                    protocol_name: self.protocol_name,
                }))
            } else {
                Poll::Pending
            }
        }
    }
    /// Poll for the response to `txid`, registering the waker associated with `cx` to be awoken,
    /// or returning the response buffer if it has been received.
    fn poll_recv_msg_response(&self, txid: Txid, cx: &mut Context<'_>) -> Poll<Result<MessageBufEtc, Error>> {
        let interest_id = InterestId::from_txid(txid);
        {
            // Register our waker with the interest if we haven't received a message yet.
            let mut message_interests = self.message_interests.lock();
            message_interests
                .get_mut(interest_id.as_raw_id())
                .expect("Polled unregistered interest")
                .register(cx);
        }
        // Process any data on the channel, registering tasks still waiting for wake when the
        // channel becomes ready.
        let epitaph = self.recv_all()?;
        let mut message_interests = self.message_interests.lock();
        if message_interests
            .get(interest_id.as_raw_id())
            .expect("Polled unregistered interest")
            .is_received()
        {
            // If we got the result remove the received buffer and return, freeing up the
            // space for a new message.
            let buf = message_interests.remove(interest_id.as_raw_id()).unwrap_received();
            Poll::Ready(Ok(buf))
        } else {
            if let Some(status) = epitaph {
                Poll::Ready(Err(Error::ClientChannelClosed {
                    status,
                    protocol_name: self.protocol_name,
                }))
            } else {
                Poll::Pending
            }
        }
    }
    /// Poll for the receipt of any response message or an event.
    /// Wakers present in any MessageInterest or the EventReceiver when this is called will be
    /// notified when their message arrives or when there is new data if the channel is empty.
    ///
    /// Returns the epitaph (or PEER_CLOSED) if the channel was closed, and None otherwise.
    fn recv_all(&self) -> Result<Option<fx_status::Status>, Error> {
        // TODO(cramertj) return errors if one has occurred _ever_ in recv_all, not just if
        // one happens on this call.
        loop {
            // Acquire a mutex so that only one thread can read from the underlying channel
            // at a time. Channel is already synchronized, but we need to also decode the
            // FIDL message header atomically so that epitaphs can be properly handled.
            let mut epitaph_lock = self.epitaph.lock();
            if epitaph_lock.is_some() {
                return Ok(*epitaph_lock);
            }
            let buf = {
                // Get a combined waker that will wake up everyone who is waiting.
                let waker = self.get_combined_waker();
                let cx = &mut Context::from_waker(&waker);
                let mut buf = MessageBufEtc::new();
                let result = self.channel.recv_etc_from(cx, &mut buf);
                match result {
                    Poll::Ready(Ok(())) => {}
                    Poll::Ready(Err(fx_status::Status::PEER_CLOSED)) => {
                        // The channel has been closed, and no epitaph was received.
                        // Set the epitaph to PEER_CLOSED.
                        *epitaph_lock = Some(fx_status::Status::PEER_CLOSED);
                        // Wake up everyone waiting, since an epitaph is broadcast to all receivers.
                        self.wake_all();
                        return Ok(*epitaph_lock);
                    }
                    Poll::Ready(Err(e)) => return Err(Error::ClientRead(e)),
                    Poll::Pending => {
                        return Ok(None);
                    }
                };
                buf
            };
            let (header, body_bytes) = decode_transaction_header(buf.bytes()).map_err(|_| Error::InvalidHeader)?;
            if !header.is_compatible() {
                return Err(Error::IncompatibleMagicNumber(header.magic_number()));
            }
            if header.is_epitaph() {
                // Received an epitaph. Record this so that everyone receives the same epitaph.
                let handles = &mut [];
                let mut epitaph_body = Decode::<EpitaphBody>::new_empty();
                Decoder::decode_into::<EpitaphBody>(&header, &body_bytes, handles, &mut epitaph_body)?;
                *epitaph_lock = Some(epitaph_body.error);
                // Wake up everyone waiting, since an epitaph is broadcast to all receivers.
                self.wake_all();
                return Ok(*epitaph_lock);
            }
            // Epitaph handling is done, so the lock is no longer required.
            drop(epitaph_lock);
            if header.tx_id() == 0 {
                // received an event
                let mut lock = self.event_channel.lock();
                lock.queue.push_back(buf);
                lock.listener.wake();
            } else {
                // received a message response
                let recvd_interest_id = InterestId::from_txid(Txid(header.tx_id()));
                // Look for a message interest with the given ID.
                // If one is found, store the message so that it can be picked up later.
                let mut message_interests = self.message_interests.lock();
                let raw_recvd_interest_id = recvd_interest_id.as_raw_id();
                // TODO(fxbug.dev/114743): Unknown transaction IDs should cause
                // an error/close the channel.
                if let Some(interest) = message_interests.get_mut(raw_recvd_interest_id) {
                    let remove = interest.receive(buf);
                    if remove {
                        message_interests.remove(raw_recvd_interest_id);
                    }
                }
            }
        }
    }
    fn deregister_msg_interest(&self, InterestId(id): InterestId) {
        let mut lock = self.message_interests.lock();
        if lock[id].is_received() {
            lock.remove(id);
        } else {
            lock[id] = MessageInterest::Discard;
        }
    }
    /// Gets a waker that will wake up all the tasks that are waiting on this channel.
    /// `wake_all` is preferred if you are certain you are waking everyone immediately, as it is
    /// idempotent (it will only wake each task once)
    // TODO(fxbug.dev/74427): if Arc::new_cyclic becomes stable, we can wake tasks only when their
    // message has arrived.
    fn get_combined_waker(&self) -> Waker {
        let mut wakers = Vec::new();
        {
            let lock = self.message_interests.lock();
            wakers.reserve(lock.len() + 1);
            for (_, message_interest) in lock.iter() {
                if let MessageInterest::Waiting(waker) = message_interest {
                    wakers.push(waker.clone());
                }
            }
        }
        {
            let lock = self.event_channel.lock();
            if let EventListener::Some(waker) = &lock.listener {
                wakers.push(waker.clone());
            }
        }
        if !wakers.is_empty() {
            wakers.shrink_to_fit();
            CombinedWaker::make_waker(wakers)
        } else {
            noop_waker()
        }
    }
    /// Wakes all tasks that have polled on this channel.
    fn wake_all(&self) {
        {
            let mut lock = self.message_interests.lock();
            for (_, interest) in lock.iter_mut() {
                interest.wake();
            }
        }
        self.event_channel.lock().listener.wake();
    }
}

pub mod sync {
    //! Synchronous MIDL Client
    use super::*;
    use {
        crate::encoding::{maybe_overflowing_after_encode, maybe_overflowing_decode},
        fiber_rust::{self as fx, AsHandleRef},
    };

    /// A synchronous client for making FIDL calls.
    #[derive(Debug)]
    pub struct Client {
        // Underlying channel
        channel: fx::Channel,

        // The `ProtocolMarker::DEBUG_NAME` for the service this client connects to.
        protocol_name: &'static str,
    }

    impl Client {
        /// Create a new synchronous FIDL client.
        pub fn new(channel: fx::Channel, protocol_name: &'static str) -> Self {
            // Initialize tracing. This is a no-op if FIDL userspace tracing is
            // disabled or if the function was already called.
            Client { channel, protocol_name }
        }

        /// Get the underlying channel out of the client.
        pub fn into_channel(self) -> fx::Channel {
            self.channel
        }

        /// Send a new message.
        pub fn send<T: TypeMarker, const OVERFLOWABLE: bool>(
            &self,
            body: impl Encode<T>,
            ordinal: u64,
            dynamic_flags: DynamicFlags,
        ) -> Result<(), Error> {
            let mut write_bytes = Vec::new();
            let mut write_handles = Vec::new();
            let msg = TransactionMessage {
                header: TransactionHeader::new(0, ordinal, dynamic_flags),
                body,
            };
            Encoder::encode::<TransactionMessageType<T>>(&mut write_bytes, &mut write_handles, msg)?;
            if OVERFLOWABLE {
                maybe_overflowing_after_encode(&mut write_bytes, &mut write_handles)?;
            }
            match self.channel.write_etc(&mut write_bytes, &mut write_handles) {
                Ok(()) | Err(fx_status::Status::PEER_CLOSED) => Ok(()),
                Err(e) => Err(Error::ClientWrite(e)),
            }
        }

        /// Send a new message expecting a response.
        pub fn send_query<
            Request: TypeMarker,
            Response: TypeMarker,
            const REQUEST_ENCODE_OVERFLOWABLE: bool,
            const RESPONSE_DECODE_OVERFLOWABLE: bool,
        >(
            &self,
            body: impl Encode<Request>,
            ordinal: u64,
            dynamic_flags: DynamicFlags,
            deadline: fx::Time,
        ) -> Result<Response::Owned, Error> {
            let mut write_bytes = Vec::new();
            let mut write_handles = Vec::new();
            let msg = TransactionMessage {
                header: TransactionHeader::new(0, ordinal, dynamic_flags),
                body,
            };
            Encoder::encode::<TransactionMessageType<Request>>(&mut write_bytes, &mut write_handles, msg)?;
            if REQUEST_ENCODE_OVERFLOWABLE {
                maybe_overflowing_after_encode(&mut write_bytes, &mut write_handles)?;
            }
            let mut buf = fx::MessageBufEtc::new();
            buf.ensure_capacity_bytes(fx::sys::FX_CHANNEL_MAX_MSG_BYTES as usize);
            buf.ensure_capacity_handle_infos(fx::sys::FX_CHANNEL_MAX_MSG_HANDLES as usize);

            // TODO: We should be able to use the same memory to back the bytes we use for writing
            // and reading.
            self.channel
                .call_etc(deadline, &write_bytes, &mut write_handles, &mut buf)
                .map_err(|e| self.wrap_error(Error::ClientCall, e))?;
            let (bytes, mut handle_infos) = buf.split();
            let (header, body_bytes) = decode_transaction_header(&bytes)?;
            let mut output = Decode::<Response>::new_empty();
            if RESPONSE_DECODE_OVERFLOWABLE {
                maybe_overflowing_decode::<Response>(&header, body_bytes, &mut handle_infos, &mut output)?;
            } else {
                Decoder::decode_into::<Response>(&header, body_bytes, &mut handle_infos, &mut output)?;
            }
            Ok(output)
        }

        /// Wait for an event to arrive on the underlying channel.
        pub fn wait_for_event(&self, deadline: fx::Time) -> Result<MessageBufEtc, Error> {
            let mut buf = fx::MessageBufEtc::new();
            buf.ensure_capacity_bytes(fx::sys::FX_CHANNEL_MAX_MSG_BYTES as usize);
            buf.ensure_capacity_handle_infos(fx::sys::FX_CHANNEL_MAX_MSG_HANDLES as usize);
            
            loop {
                self.channel
                    .wait_handle(
                        fx::Signals::CHANNEL_READABLE | fx::Signals::CHANNEL_PEER_CLOSED,
                        deadline,
                    )
                    .map_err(|e| self.wrap_error(Error::ClientEvent, e))?;
                match self.channel.read_etc(&mut buf) {
                    Ok(()) => {
                        // We succeeded in reading the message. Check that it is
                        // an event not a two-way method reply.
                        let (header, _) = decode_transaction_header(buf.bytes()).map_err(|_| Error::InvalidHeader)?;
                        if header.tx_id() != 0 {
                            return Err(Error::UnexpectedSyncResponse);
                        }
                        return Ok(buf);
                    }
                    Err(fx::Status::SHOULD_WAIT) => {
                        // Some other thread read the message we woke up to read.
                        continue;
                    }
                    Err(e) => {
                        return Err(self.wrap_error(Error::ClientRead, e));
                    }
                }
            }
        }

        /// Wraps an error in the given `variant` of the `Error` enum, except
        /// for `zx_status::Status::PEER_CLOSED`, in which case it uses the
        /// `Error::ClientChannelClosed` variant.
        fn wrap_error<T: Fn(fx_status::Status) -> Error>(&self, variant: T, err: fx_status::Status) -> Error {
            if err == fx_status::Status::PEER_CLOSED {
                Error::ClientChannelClosed {
                    status: fx_status::Status::PEER_CLOSED,
                    protocol_name: self.protocol_name,
                }
            } else {
                variant(err)
            }
        }
    }
}

// TODO: tests
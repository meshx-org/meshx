
// WARNING: This file is machine generated by midlgen.

#![allow(
    unused_parens, // one-element-tuple-case is not a tuple
    unused_mut, // not all args require mutation, but many do
    nonstandard_style, // auto-caps does its best, but is not always successful
)]

#![recursion_limit="512"]

#[allow(unused_imports)]
use fiber as fx;

#[allow(unused_imports)]
use {
    bitflags::bitflags,
    fuchsia_zircon_status as zx_status,
    futures::future::{self, MaybeDone, TryFutureExt},
    midl::{
        midl_bits,
        midl_empty_struct,
        midl_enum,
        midl_struct_copy,
        midl_struct,
        midl_table,
        midl_union,
        wrap_handle_metadata,
        encoding::{Encodable as _, Decodable as _, zerocopy},
        endpoints::{ControlHandle as _, Responder as _},
        client::{
            QueryResponseFut,
            decode_transaction_body_fut,
        },
    },
};

const _MIDL_TRACE_BINDINGS_RUST: u32 = 6;


pub const TEST_CONST: u32 = 123;

#[derive(Copy, Clone, Debug, Eq, PartialEq, Ord, PartialOrd, Hash)]
#[non_exhaustive]
pub enum TestEnum {
}


impl TestEnum {
    #[inline]
    pub fn from_primitive(prim: u16) -> Option<Self> {
        match prim {
            _ => None,
        }
    }

    #[inline]
    pub fn from_primitive_allow_unknown(prim: u16) -> Self {
        match prim {
            #[allow(deprecated)]
            x => Self::__Unknown(x),
        }
    }

    #[inline]
    pub fn unknown() -> Self {
        #[allow(deprecated)]
        Self::__Unknown()
    }

    #[inline]
    pub const fn into_primitive(self) -> u16 {
        match self {
            #[allow(deprecated)]
            Self::__Unknown(x) => x,
        }
    }

    #[inline]
    pub fn validate(self) -> std::result::Result<Self, u16> {
        match self {
            #[allow(deprecated)]
            Self::__Unknown(x) => Err(x),
        }
    }

    #[inline]
    pub fn is_unknown(&self) -> bool {
        self.validate().is_err()
    }
}

fidl_enum! {
    name: TestEnum,
    prim_ty: u16,
    strict: true,
    min_member: default,
}


#[repr(C)]
pub struct TestSctruct {
    pub test_member: u32,
}


fidl_struct_copy! {
    name: TestSctruct,
    members: [
        test_member {
            ty: u32,
            offset_v1: 0,
            offset_v2: 0,
        },
    ],
    padding_v1: [
        ],
    padding_v2: [
    ],
    size_v1: 12,
    size_v2: 12,
    align_v1: 12,
    align_v2: 12,
}

 






#[derive(Debug, Copy, Clone, Eq, PartialEq, Ord, PartialOrd, Hash)]
pub struct TestProtocolMarker;

impl fidl::endpoints::ProtocolMarker for TestProtocolMarker {
    type Proxy = TestProtocolProxy;
    type RequestStream = TestProtocolRequestStream;
    const DEBUG_NAME: &'static str = "TestProtocol";
}

impl fidl::endpoints::DiscoverableProtocolMarker for TestProtocolMarker {}

pub trait TestProtocolProxyInterface: Send + Sync {
}

#[derive(Debug)]
#[cfg(target_os = "fuchsia")]
pub struct TestProtocolSynchronousProxy {
    client: fidl::client::sync::Client,
}

#[cfg(target_os = "fuchsia")]
impl TestProtocolSynchronousProxy {
    pub fn new(channel: fidl::Channel) -> Self {
        let protocol_name = <TestProtocolMarker as fidl::endpoints::ProtocolMarker>::DEBUG_NAME;
        Self { client: fidl::client::sync::Client::new(channel, protocol_name) }
    }

    pub fn into_channel(self) -> fidl::Channel {
        self.client.into_channel()
    }

    /// Waits until an event arrives and returns it. It is safe for other
    /// threads to make concurrent requests while waiting for an event.
    pub fn wait_for_event(&self, deadline: zx::Time) -> Result<TestProtocolEvent, fidl::Error> {
        TestProtocolEvent::decode(self.client.wait_for_event(deadline)?)
    }

}

#[derive(Debug, Clone)]
pub struct TestProtocolProxy {
    client: fidl::client::Client,
}

impl fidl::endpoints::Proxy for TestProtocolProxy {
    type Protocol = TestProtocolMarker;

    fn from_channel(inner: fidl::AsyncChannel) -> Self {
        Self::new(inner)
    }

    fn into_channel(self) -> Result<::fidl::AsyncChannel, Self> {
        self.client.into_channel().map_err(|client| Self { client })
    }

    fn as_channel(&self) -> &::fidl::AsyncChannel {
        self.client.as_channel()
    }
}

impl TestProtocolProxy {
    /// Create a new Proxy for TestProtocol
    pub fn new(channel: fidl::AsyncChannel) -> Self {
        let protocol_name = <TestProtocolMarker as fidl::endpoints::ProtocolMarker>::DEBUG_NAME;
        Self { client: fidl::client::Client::new(channel, protocol_name) }
    }

    /// Get a Stream of events from the remote end of the TestProtocol protocol
    ///
    /// # Panics
    ///
    /// Panics if the event stream was already taken.
    pub fn take_event_stream(&self) -> TestProtocolEventStream {
        TestProtocolEventStream {
            event_receiver: self.client.take_event_receiver(),
        }
    }

}

impl TestProtocolProxyInterface for TestProtocolProxy {
}

pub struct TestProtocolEventStream {
    event_receiver: fidl::client::EventReceiver,
}

impl std::marker::Unpin for TestProtocolEventStream {}

impl futures::stream::FusedStream for TestProtocolEventStream {
    fn is_terminated(&self) -> bool {
        self.event_receiver.is_terminated()
    }
}

impl futures::Stream for TestProtocolEventStream {
    type Item = Result<TestProtocolEvent, fidl::Error>;

    fn poll_next(mut self: std::pin::Pin<&mut Self>, cx: &mut std::task::Context<'_>)
        -> std::task::Poll<Option<Self::Item>>
    {
        let buf = match futures::ready!(
            futures::stream::StreamExt::poll_next_unpin(&mut self.event_receiver, cx)?
        ) {
            Some(buf) => buf,
            None => return std::task::Poll::Ready(None),
        };

        std::task::Poll::Ready(Some(TestProtocolEvent::decode(buf)))
    }
}

#[derive(Debug)]
pub enum TestProtocolEvent {

}

impl TestProtocolEvent {

    /// Decodes a message buffer as a [`TestProtocolEvent`]. Transaction
    /// ID in the message must be zero; this method does not check TXID.
    fn decode(mut buf: fidl::MessageBufEtc) -> Result<TestProtocolEvent, fidl::Error> {
        let (bytes, _handles) = buf.split_mut();
        let (tx_header, _body_bytes) = fidl::encoding::decode_transaction_header(bytes)?;

        match tx_header.ordinal() {
            _ => Err(fidl::Error::UnknownOrdinal {
                ordinal: tx_header.ordinal(),
                protocol_name: <TestProtocolMarker as fidl::endpoints::ProtocolMarker>::DEBUG_NAME,
            })
        }
    }
}

/// A Stream of incoming requests for TestProtocol
pub struct TestProtocolRequestStream {
    inner: std::sync::Arc<fidl::ServeInner>,
    is_terminated: bool,
}

impl std::marker::Unpin for TestProtocolRequestStream {}

impl futures::stream::FusedStream for TestProtocolRequestStream {
    fn is_terminated(&self) -> bool {
        self.is_terminated
    }
}

impl fidl::endpoints::RequestStream for TestProtocolRequestStream {
    type Protocol = TestProtocolMarker;
    type ControlHandle = TestProtocolControlHandle;

    fn from_channel(channel: fidl::AsyncChannel) -> Self {
        Self {
            inner: std::sync::Arc::new(fidl::ServeInner::new(channel)),
            is_terminated: false,
        }
    }

    fn control_handle(&self) -> Self::ControlHandle {
        TestProtocolControlHandle { inner: self.inner.clone() }
    }

    fn into_inner(self) -> (::std::sync::Arc<fidl::ServeInner>, bool) {
        (self.inner, self.is_terminated)
    }

    fn from_inner(inner: std::sync::Arc<fidl::ServeInner>, is_terminated: bool)
        -> Self
    {
        Self { inner, is_terminated }
    }
}

impl futures::Stream for TestProtocolRequestStream {
    type Item = Result<TestProtocolRequest, fidl::Error>;

    fn poll_next(mut self: std::pin::Pin<&mut Self>, cx: &mut std::task::Context<'_>)
        -> std::task::Poll<Option<Self::Item>>
    {
        let this = &mut *self;
        if this.inner.poll_shutdown(cx) {
            this.is_terminated = true;
            return std::task::Poll::Ready(None);
        }
        if this.is_terminated {
            panic!("polled TestProtocolRequestStream after completion");
        }
        fidl::encoding::with_tls_decode_buf(|bytes, handles| {
            match this.inner.channel().read_etc(cx, bytes, handles) {
                std::task::Poll::Ready(Ok(())) => {},
                std::task::Poll::Pending => return std::task::Poll::Pending,
                std::task::Poll::Ready(Err(zx_status::Status::PEER_CLOSED)) => {
                    this.is_terminated = true;
                    return std::task::Poll::Ready(None);
                }
                std::task::Poll::Ready(Err(e)) => return std::task::Poll::Ready(Some(Err(fidl::Error::ServerRequestRead(e)))),
            }

            // A message has been received from the channel
            let (header, _body_bytes) = fidl::encoding::decode_transaction_header(bytes)?;
            if !header.is_compatible() {
                return std::task::Poll::Ready(Some(Err(fidl::Error::IncompatibleMagicNumber(header.magic_number()))));
            }

            std::task::Poll::Ready(Some(match header.ordinal() {
                
                _ => Err(fidl::Error::UnknownOrdinal {
                    ordinal: header.ordinal(),
                    protocol_name: <TestProtocolMarker as fidl::endpoints::ProtocolMarker>::DEBUG_NAME,
                }),
            }))
        })
    }
}

#[derive(Debug)]
pub enum TestProtocolRequest {

}

impl TestProtocolRequest {

    /// Name of the method defined in FIDL
    pub fn method_name(&self) -> &'static str {
        match *self {

        }
    }
}

#[derive(Debug, Clone)]
pub struct TestProtocolControlHandle {
    inner: std::sync::Arc<fidl::ServeInner>,
}

impl fidl::endpoints::ControlHandle for TestProtocolControlHandle {
    fn shutdown(&self) {
        self.inner.shutdown()
    }

    fn shutdown_with_epitaph(&self, status: zx_status::Status) {
        self.inner.shutdown_with_epitaph(status)
    }
}

impl TestProtocolControlHandle {
}



 
use std::collections::VecDeque;
use std::rc::Rc;
use std::sync::Arc;
use std::{any::Any, sync::RwLock};

use fiber_sys::{self as sys, fx_status_t};

use super::{
    BaseDispatcher, Dispatcher, GenericDispatcher, KernelHandle, MessagePacketPtr, PeerHolder, PeeredDispatcher,
    PeeredDispatcherBase, TypedDispatcher,
};

// This value is part of the zx_channel_call contract.
const MIN_KERNEL_GENERATED_TXID: u32 = 0x80000000;

fn is_kernel_generated_txid(txid: sys::fx_txid_t) -> bool {
    return txid >= MIN_KERNEL_GENERATED_TXID;
}

// MessageWaiter's state is guarded by the lock of the
// owning ChannelDispatcher, and Deliver(), Signal(), Cancel(),
// and EndWait() methods must only be called under
// that lock.
//
// MessageWaiters are embedded in ThreadDispatchers, and the channel_ pointer
// can only be manipulated by their thread (via BeginWait() or EndWait()), and
// only transitions to nullptr while holding the ChannelDispatcher's lock.
//
// See also: comments in ChannelDispatcher::Call()
#[derive(Debug)]
struct MessageWaiter {
    channel: Option<Arc<ChannelDispatcher>>,
    msg: Option<MessagePacketPtr>,

    // TODO(teisenbe/swetland): Investigate hoisting this outside to reduce
    // userthread size
    // event: Event;
    txid: sys::fx_txid_t,
    status: sys::fx_status_t,
}

impl MessageWaiter {
    // public:
    fn new() -> Self {
        Self {
            txid: 0,
            status: sys::FX_ERR_BAD_STATE,
            channel: None,
            msg: None,
        }
    }

    fn begin_wait(&self, channel: Rc<ChannelDispatcher>) -> sys::fx_status_t {
        unimplemented!()
    }
    fn deliver(&self, msg: MessagePacketPtr) {
        unimplemented!()
    }
    fn cancel(&self, status: sys::fx_status_t) {
        unimplemented!()
    }
    fn wait(&self /*deadline: &Deadline*/) -> sys::fx_status_t {
        unimplemented!()
    }
    // Returns any delivered message via out and the status.
    fn end_wait(&self, out: *mut MessagePacketPtr) -> sys::fx_status_t {
        unimplemented!()
    }

    fn get_channel(&self) -> Option<Arc<ChannelDispatcher>> {
        self.channel.clone()
    }

    fn get_txid(&self) -> sys::fx_txid_t {
        return self.txid;
    }

    fn set_txid(&mut self, txid: sys::fx_txid_t) {
        self.txid = txid;
    }
}

#[derive(Debug)]
struct ChannelGuardedState {
    waiters: Vec<MessageWaiter>,
    messages: VecDeque<MessagePacketPtr>,
    max_message_count: u32,

    txid: u32, // TA_GUARDED(get_lock()) = 0;

    /// True if the this object's peer has been closed. This field exists so that
    /// |Read| can check for peer closed without having to acquire |get_lock()|.
    peer_has_closed: bool, // TA_GUARDED(channel_lock_) = false;
}

#[derive(Debug)]
pub(crate) struct ChannelDispatcher {
    base: BaseDispatcher,
    peered_base: PeeredDispatcherBase<ChannelDispatcher>,

    guarded: RwLock<ChannelGuardedState>,

    /// Tracks the process that is allowed to issue calls, for example write
    /// to the opposite end. Without it, one can see writes out of order with
    /// respect of the previous and current owner. We avoid locking and updating
    /// the |owner_| if the new owner is kernel, which happens when the endpoint
    /// is written into a channel or during process destruction.
    ///
    /// The locking protocol for this field is a little tricky.  The Read method,
    /// which only ever acquires the channel_lock_, must read this field.  The
    /// Write method also needs to read this field, however, it needs to do so
    /// before it would otherwise need to acquire the channel_lock_.  So to avoid
    /// having Write prematurely acquire and release the channel_lock_, we instead
    /// require that either |get_lock()| or channel_lock_ are held when reading
    /// this field and both are held when writing it.
    owner: sys::fx_koid_t, //= ZX_KOID_INVALID;
}

impl Dispatcher for ChannelDispatcher {
    fn get_koid(&self) -> sys::fx_koid_t {
        self.base.get_koid()
    }

    fn get_related_koid(&self) -> sys::fx_koid_t {
        0
    }

    fn base(&self) -> &BaseDispatcher {
        &self.base
    }
}

impl PeeredDispatcher for ChannelDispatcher {
    /// Initialize this dispatcher's peer field.
    ///
    /// This method is logically part of the class constructor and must be called exactly once, during
    /// initialization, prior to any other thread obtaining a reference to the object.  These
    /// constraints allow for an optimization where fields are accessed without acquiring the lock,
    /// hence the TA_NO_THREAD_SAFETY_ANALYSIS annotation.
    fn init_peer(&self, peer: Arc<Self>) {
        let mut peered_state = self.peered_base.guarded.lock().unwrap();

        debug_assert!(peered_state.peer.is_none());
        debug_assert!(peered_state.peer_koid.is_none());

        peered_state.peer_koid = Some(peer.get_koid());
        peered_state.peer = Some(peer);
    }

    fn peer(&self) -> Option<Arc<Self>> {
        let peered_state = self.peered_base.guarded.lock().unwrap();
        peered_state.peer.clone()
    }
}

impl TypedDispatcher for ChannelDispatcher {
    fn default_rights() -> sys::fx_rights_t {
        sys::FX_DEFAULT_CHANNEL_RIGHTS
    }

    fn get_type() -> sys::fx_obj_type_t {
        sys::FX_OBJ_TYPE_CHANNEL
    }
}

impl ChannelDispatcher {
    pub(crate) fn create() -> Result<
        (
            KernelHandle<ChannelDispatcher>,
            KernelHandle<ChannelDispatcher>,
            sys::fx_rights_t,
        ),
        sys::fx_status_t,
    > {
        let holder0: Arc<PeerHolder<ChannelDispatcher>> = Arc::new(PeerHolder::new());
        let holder1 = holder0.clone();

        let new_kernel_handle0 =
            KernelHandle::new(GenericDispatcher::ChannelDispatcher(ChannelDispatcher::new(holder0)));
        let new_kernel_handle1 =
            KernelHandle::new(GenericDispatcher::ChannelDispatcher(ChannelDispatcher::new(holder1)));

        let new_handle0 = new_kernel_handle0.dispatcher().as_channel_dispatcher().unwrap();
        let new_handle1 = new_kernel_handle1.dispatcher().as_channel_dispatcher().unwrap();

        new_handle0.init_peer(new_handle1.clone());
        new_handle1.init_peer(new_handle0);

        let rights = ChannelDispatcher::default_rights();
        //let handle0 = new_handle0;
        //let handle1 = new_handle1;

        Ok((new_kernel_handle0, new_kernel_handle1, rights))
    }

    fn new(peer: Arc<PeerHolder<ChannelDispatcher>>) -> Arc<Self> {
        let channel = Arc::new(ChannelDispatcher {
            base: BaseDispatcher::new(0),
            peered_base: PeeredDispatcherBase::new(peer),
            owner: sys::FX_KOID_INVALID,
            guarded: RwLock::new(ChannelGuardedState {
                waiters: Vec::new(),
                messages: VecDeque::new(),
                max_message_count: 0,
                txid: 0,
                peer_has_closed: false,
            }),
        });

        channel
    }

    // This is called before either ChannelDispatcher is accessible from threads other than the one
    // initializing the channel, so it does not need locking
    //fn init(&self, other: Arc<ChannelDispatcher>) {
    //    self.peered_base.peer = Some(other);
    //    self.peered_base.peer_koid = self.peered_base.peer.get_koid();
    //}

    /// Generate a unique txid to be used in a channel call.
    fn generate_txid() -> sys::fx_txid_t {
        unimplemented!()
    }

    /// Write to the opposing endpoint's message queue. |owner| is the handle table koid of the process
    /// attempting to write to the channel, or FX_KOID_INVALID if kernel is doing it.
    pub(crate) fn write(&self, owner: sys::fx_koid_t, msg: MessagePacketPtr) -> sys::fx_status_t {
        // canary_.Assert();
        // Guard<CriticalMutex> guard{get_lock()};

        // Failing this test is only possible if this process has two threads racing:
        // one thread is issuing channel_write() and one thread is moving the handle
        // to another process.
        if owner != self.owner {
            return sys::FX_ERR_BAD_HANDLE;
        }

        if self.peer().is_none() {
            return sys::FX_ERR_PEER_CLOSED;
        }

        let peer = self.peer().unwrap();

        // AssertHeld(*self.peer().get_lock());

        let result = peer.try_write_to_message_waiter(msg);

        if result.is_ok() {
            return sys::FX_OK;
        }

        peer.write_self(result.unwrap_err());
        sys::FX_OK
    }

    fn write_self(&self, msg: MessagePacketPtr) {
        //canary_.Assert();

        // Once we've acquired the channel_lock_ we're going to make a copy of the previously active
        // signals and raise the READABLE signal before dropping the lock. After we've dropped the lock,
        // we'll notify observers using the previously active signals plus READABLE.
        //
        // There are several things to note about this sequence:
        //
        // 1. We must hold channel_lock_ while updating the stored signals (RaiseSignalsLocked) to
        // synchronize with thread adding, removing, or canceling observers otherwise we may create a
        // spurious READABLE signal (see NoSpuriousReadableSignalWhenRacing test).
        //
        // 2. We must release the channel_lock_ before notifying observers to ensure that Read can execute
        // concurrently with NotifyObserversLocked, which is a potentially long running call.
        //
        // 3. We can skip the call to NotifyObserversLocked if the previously active signals contained
        // READABLE (because there can't be any observers still waiting for READABLE if that signal is
        // already active).
        let previous_signals: sys::fx_signals_t;

        {
            let mut lock = self.guarded.write().unwrap();
            lock.messages.push_back(msg);
            previous_signals = self.base().raise_signals_locked(sys::FX_CHANNEL_READABLE);
            let size = lock.messages.len() as u32;
            if size > lock.max_message_count {
                lock.max_message_count = size;
            }
        }

        // Don't bother waking observers if FX_CHANNEL_READABLE was already active.
        if (previous_signals & sys::FX_CHANNEL_READABLE) == 0 {
            self.base()
                .notify_observers_locked(previous_signals | sys::FX_CHANNEL_READABLE);
        }
    }

    fn try_write_to_message_waiter(&self, msg: MessagePacketPtr) -> Result<(), MessagePacketPtr> {
        let lock = self.guarded.read().unwrap();

        if lock.waiters.is_empty() {
            return Err(msg);
        }

        // If the far side has "call" waiters waiting for replies, see if this message's txid matches one
        // of them.  If so, deliver it.  Note, because callers use a kernel generated txid we can skip
        // checking the list if this message's txid isn't kernel generated.
        let txid = msg.get_txid();
        if !is_kernel_generated_txid(txid) {
            return Err(msg);
        }

        for waiter in lock.waiters.iter() {
            // (3C) Deliver message to waiter.
            // Remove waiter from list.
            if waiter.get_txid() == txid {
                // TODO: self.waiters.erase(waiter);
                waiter.deliver(msg);
                return Ok(());
            }
        }

        Err(msg)
    }

    // This method should never acquire |get_lock()|.  See the comment at |channel_lock_| for details.
    pub fn read(
        &self,
        owner: sys::fx_koid_t,
        msg_size: &mut usize,
        msg_handle_count: &mut u32,
        msg: &mut MessagePacketPtr,
        may_discard: bool,
    ) -> sys::fx_status_t {
        //canary_.Assert();

        let max_size = *msg_size;
        let max_handle_count = *msg_handle_count;

        let mut guard = self.guarded.write().unwrap();

        if (owner != self.owner) {
            return sys::FX_ERR_BAD_HANDLE;
        }

        if (guard.messages.is_empty()) {
            return if guard.peer_has_closed {
                sys::FX_ERR_PEER_CLOSED
            } else {
                sys::FX_ERR_SHOULD_WAIT
            };
        }

        let front = guard.messages.front().unwrap();

        *msg_size = front.data_size();
        *msg_handle_count = front.num_handles();

        let mut status: sys::fx_status_t = sys::FX_OK;

        if (*msg_size > max_size || *msg_handle_count > max_handle_count) {
            if (!may_discard) {
                return sys::FX_ERR_BUFFER_TOO_SMALL;
            }
            status = sys::FX_ERR_BUFFER_TOO_SMALL;
        }

        *msg = guard.messages.pop_front().unwrap();
        if (guard.messages.is_empty()) {
            todo!()
            // ClearSignals(ZX_CHANNEL_READABLE);
        }

        status
    }
}

// Copyright 2016 The Fuchsia Authors
//
// Use of this source code is governed by a MIT-style
// license that can be found in the LICENSE file or at
// https://opensource.org/licenses/MIT

use super::signal_observer::SignalObserver;
use crate::koid;
use fiber_sys as sys;
use std::marker::PhantomData;
use std::sync::atomic::{fence, AtomicU32, Ordering};
use std::sync::{Arc, Mutex, RwLock};

#[derive(Debug)]
struct GuardedDispatcherState {
    // List of observers watching for changes in signals on this dispatcher.
    observers: Vec<Box<dyn SignalObserver + Send + Sync>>,
}

#[derive(Debug)]
pub(crate) struct BaseDispatcher {
    koid: sys::fx_koid_t,
    handle_count: AtomicU32,

    guarded: RwLock<GuardedDispatcherState>, // TA_GUARDED(get_lock());

    // |signals| is the set of currently active signals.
    //
    // There are several high-level operations in which the signal state is accessed.  Some of these
    // operations require holding |get_lock()| and some do not.  See the comment at |get_lock()|.
    //
    // 1. Adding, removing, or canceling an observer - These operations involve access to both
    // signals_ and observers_ and must be performed while holding get_lock().
    //
    // 2. Updating signal state - This is a composite operation consisting of two sub-operations:
    //
    //    a. Clearing signals - Because no observer may be triggered by deasserting (clearing) a
    //    signal, it is not necessary to hold |get_lock()| while clearing.  Simply clearing signals
    //    does not need to access observers_.
    //
    //    b. Raising (setting) signals and notifying matched observers - This operation must appear
    //    atomic to and cannot overlap with any of the operations in #1 above.  |get_lock()| must be
    //    held for the duration of this operation.
    //
    // Regardless of whether the operation requires holding |get_lock()| or not, access to this field
    // should use acquire/release memory ordering.  That is, use memory_order_acquire for read,
    // memory_order_release for write, and memory_order_acq_rel for read-modify-write.  To understand
    // why it's important to use acquire/release, consider the following (contrived) example:
    //
    //   RelaxedAtomic<bool> ready;
    //
    //   void T1() {
    //     // Wait for T2 to clear the signals.
    //     while (d.PollSignals() & kMask) {
    //     }
    //     // Now that we've seen there are no signals we can be confident that ready is true.
    //     ASSERT(ready.load());
    //   }
    //
    //   void T2() {
    //     ready.store(true);
    //     d.ClearSignals(kMask);
    //   }
    //
    // In the example above, T1's ASSERT may fire if PollSignals or ClearSignals were to use relaxed
    // memory order for accessing signals_.
    signals: AtomicU32, // alias fx_signals_t
}

impl BaseDispatcher {
    pub(super) fn new(signals: sys::fx_signals_t) -> Self {
        // kcounter_add(dispatcher_create_count, 1);
        BaseDispatcher {
            koid: koid::generate(),
            handle_count: AtomicU32::new(0),
            signals: AtomicU32::new(signals),
            guarded: RwLock::new(GuardedDispatcherState { observers: Vec::new() }),
        }
    }

    pub(super) fn get_koid(&self) -> sys::fx_koid_t {
        self.koid
    }

    pub(super) fn increment_handle_count(&self) {
        // As this function does not return anything actionable, not even something implicit like "you
        // now have the lock", there are no correct assumptions the caller can make about orderings
        // of this increment and any other memory access. As such it can just be relaxed.
        self.handle_count.fetch_add(1, Ordering::Relaxed);
    }

    // Returns true exactly when the handle count goes to zero.
    pub(super) fn decrement_handle_count(&self) -> bool {
        if self.handle_count.fetch_sub(1, Ordering::Release) == 1 {
            // The decrement operation above synchronizes with the fence below.  This ensures that changes
            // to the object prior to its handle count reaching 0 will be visible to the thread that
            // ultimately drops the count to 0.  This is similar to what's done in
            // |fbl::RefCountedInternal|.

            fence(Ordering::Acquire);
            return true;
        }

        false
    }

    pub(super) fn current_handle_count(&self) -> u32 {
        // Requesting the count is fundamentally racy with other users of the dispatcher. A typical
        // reference count implementation might place an acquire here for the scenario where you then
        // run an object destructor without acquiring any locks. As a handle count is not a refcount
        // and a low handle count does not imply any ownership of the dispatcher (which has its own
        // refcount), this can just be relaxed.
        self.handle_count.load(Ordering::Relaxed)
    }

    /// Raise (set) signals specified by |signals| without notifying observers.
    ///
    /// Returns the old value.
    pub(super) fn raise_signals_locked(&self, signals: sys::fx_signals_t) -> sys::fx_signals_t {
        self.signals.fetch_or(signals, Ordering::AcqRel)
    }

    /// Notify the observers waiting on one or more |signals|.
    ///
    /// unlike UpdateState and UpdateStateLocked, this method does not modify the stored signal state.
    pub(super) fn notify_observers_locked(&self, signals: sys::fx_signals_t) {
        let mut i = 0;
        let read_lock = self.guarded.read().unwrap();

        for it in read_lock.observers.iter() {
            // Ignore observers that don't need to be notified.
            if (it.get_triggering_signals() & signals) == 0 {
                i += 1;
                continue;
            }

            let to_remove = it;
            i += 1;

            let mut write_lock = self.guarded.write().unwrap();
            write_lock.observers.remove(i);
            to_remove.on_match(signals);
        }
    }
}

// PeeredDispatchers have opposing endpoints to coordinate state
// with. For example, writing into one endpoint of a Channel needs to
// modify zx_signals_t state (for the readability bit) on the opposite
// side. To coordinate their state, they share a mutex, which is held
// by the PeerHolder. Both endpoints have a RefPtr back to the
// PeerHolder; no one else ever does.
// Thus creating a pair of peered objects will typically look
// something like
//     // Make the two RefPtrs for each endpoint's handle to the mutex.
//     auto holder0 = AdoptRef(new PeerHolder<Foo>(...));
//     auto holder1 = peer_holder0;
//     // Create the opposing sides.
//     auto foo0 = AdoptRef(new Foo(ktl::move(holder0, ...));
//     auto foo1 = AdoptRef(new Foo(ktl::move(holder1, ...));
//     // Initialize the opposing sides, teaching them about each other.
//     foo0->Init(&foo1);
//     foo1->Init(&foo0);
// A PeeredDispatcher object, in its |on_zero_handles| call must clear
// out its peer's |peer_| field. This is needed to avoid leaks, and to
// ensure that |user_signal| can correctly report ZX_ERR_PEER_CLOSED.
// TODO(kulakowski) We should investigate turning this into one
// allocation. This would mean PeerHolder would have two EndPoint
// members, and that PeeredDispatcher would have custom refcounting.
#[derive(Debug)]
pub(crate) struct PeerHolder<T> {
    phantom: PhantomData<T>,
    mutex: Mutex<()>,
}

impl<T> PeerHolder<T> {
    pub(crate) fn new() -> Self {
        PeerHolder {
            phantom: PhantomData,
            mutex: Mutex::new(()),
        }
    }
}

#[derive(Debug)]
pub(crate) struct PeeredDispatcherBase<T> {
    holder: Arc<PeerHolder<T>>,

    pub peer: Option<Arc<RwLock<T>>>,
    pub peer_koid: Option<sys::fx_koid_t>,
}

impl<T> PeeredDispatcherBase<T> {
    pub(super) fn new(holder: Arc<PeerHolder<T>>) -> Self {
        PeeredDispatcherBase {
            holder,
            peer: None,
            peer_koid: None,
        }
    }

    //pub(crate) fn init_peer(&mut self, peer: Arc<RwLock<T>>) {
    //    self.peer = Some(peer);
    //    self.peer_koid = Some(peer.read().unwrap().get_koid());
    //}

    pub(crate) fn peer(&self) -> &Option<Arc<RwLock<T>>> {
        &self.peer
    }
}

pub(crate) trait TypedDispatcher {
    fn get_type() -> sys::fx_obj_type_t;
    fn default_rights() -> sys::fx_rights_t;
}

pub(crate) trait PeeredDispatcher: Dispatcher {
    fn init_peer(&mut self, peer: Arc<RwLock<Self>>);
    fn peer(&self) -> &Option<Arc<RwLock<Self>>>;
}

pub(crate) trait Dispatcher: Send + Sync {
    fn get_koid(&self) -> sys::fx_koid_t;
    fn get_related_koid(&self) -> sys::fx_koid_t;

    fn on_zero_handles(&self) {
        unreachable!()
    }

    fn is_waitable() -> bool
    where
        Self: Sized,
    {
        false
    }

    fn base(&self) -> &BaseDispatcher;
}

pub(super) trait INamed {
    // set_name() will truncate to ZX_MAX_NAME_LEN - 1 and ensure there is a
    // terminating null
    fn set_name(&self, name: String) -> sys::fx_status_t {
        return sys::FX_ERR_NOT_SUPPORTED;
    }

    // get_name() will return a null-terminated name of ZX_MAX_NAME_LEN - 1 or fewer
    // characters.  For objects that don't have names it will be "".
    fn get_name(&self) -> String {
        String::new()
    }
}

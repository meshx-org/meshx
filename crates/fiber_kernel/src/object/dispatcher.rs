use fiber_sys as sys;
use std::sync::atomic::{fence, AtomicU32, Ordering};

#[derive(Debug)]
pub struct Dispatcher {
    koid: sys::fx_koid_t,
    handle_count: AtomicU32,
}

impl Dispatcher {
    fn increment_handle_count(&self) {
        // As this function does not return anything actionable, not even something implicit like "you
        // now have the lock", there are no correct assumptions the caller can make about orderings
        // of this increment and any other memory access. As such it can just be relaxed.
        self.handle_count.fetch_add(1, Ordering::Relaxed);
    }

    // Returns true exactly when the handle count goes to zero.
    fn decrement_handle_count(&self) -> bool {
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

    fn current_handle_count(&self) -> u32 {
        // Requesting the count is fundamentally racy with other users of the dispatcher. A typical
        // reference count implementation might place an acquire here for the scenario where you then
        // run an object destructor without acquiring any locks. As a handle count is not a refcount
        // and a low handle count does not imply any ownership of the dispatcher (which has its own
        // refcount), this can just be relaxed.
        self.handle_count.load(Ordering::Relaxed)
    }
}

pub(crate) trait IDispatcher {
    fn get_type() -> sys::fx_obj_type_t;
    fn default_rights() -> sys::fx_rights_t;

    fn get_koid(&self) -> sys::fx_koid_t;
    fn get_related_koid(&self) -> sys::fx_koid_t;
}

pub(crate) trait INamed {
    // set_name() will truncate to ZX_MAX_NAME_LEN - 1 and ensure there is a
    // terminating null
    fn set_name(&self, name: String) -> sys::fx_status_t {
        return sys::FX_ERR_NOT_SUPPORTED;
    }

    // get_name() will return a null-terminated name of ZX_MAX_NAME_LEN - 1 or fewer
    // characters.  For objects that don't have names it will be "".
    fn get_name(&self) -> String;
}

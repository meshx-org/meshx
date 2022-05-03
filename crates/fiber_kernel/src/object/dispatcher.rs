use fiber_sys::fx_koid_t;
use std::sync::atomic::{fence, AtomicU32, Ordering};

#[derive(Debug)]
pub struct Dispatcher {
    koid: fx_koid_t,
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
        return false;
    }
}

use std::sync::Arc;

use fiber_sys as sys;

use super::Handle;

/// SignalObserver implementations may register to be called when
/// a signal becomes active on a particular Dispatcher.
///
/// Implementations must be thread compatible, but need not be thread safe.
pub(crate) trait SignalObserver: std::fmt::Debug + Send {
    // Called when the set of active signals matches an expected set.
    //
    // At the time this is call, it is safe to delete this object: the
    // caller will not interact with it again.
    //
    // WARNING: This is called under Dispatcher's lock
    fn on_match(&self, signals: sys::fx_signals_t);

    // Called when the registered handle (which refers to a handle to the
    // Dispatcher object) is being destroyed/"closed"/transferred. (The
    // object itself may also be destroyed shortly afterwards.)
    //
    // At the time this is call, it is safe to delete this object: the
    // caller will not interact with it again.
    //
    // WARNING: This is called under Dispatcher's lock
    fn on_cancel(&self, signals: sys::fx_signals_t);

    fn get_triggering_signals(&self) -> sys::fx_signals_t;

    fn set_triggeting_signals(&self, signals: sys::fx_signals_t);
    fn set_handle(&self, handle: Arc<Handle>);

    fn get_koid(&self) -> sys::fx_koid_t;
}

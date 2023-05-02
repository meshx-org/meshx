use fiber_sys as sys;

/// SignalObserver implementations may register to be called when
/// a signal becomes active on a particular Dispatcher.
///
/// Implementations must be thread compatible, but need not be thread safe.
pub(crate) trait SignalObserver<T>: std::fmt::Debug + core::cmp::PartialEq<T> {
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
}

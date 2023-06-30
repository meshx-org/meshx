use std::sync::Arc;

use fiber_rust::sys;

use super::{JobDispatcher, SignalObserver};

#[derive(Debug)]
pub(crate) struct RootJobObserver {
    root_job: Arc<JobDispatcher>,
}

impl RootJobObserver {
    pub(crate) fn new(root_job: Arc<JobDispatcher>) -> Self {
        Self { root_job }
    }
}

impl SignalObserver for RootJobObserver {
    fn on_match(&self, signals: sys::fx_signals_t) {
        unimplemented!()
    }

    fn on_cancel(&self, signals: sys::fx_signals_t) {
        unimplemented!()
    }

    fn get_triggering_signals(&self) -> sys::fx_signals_t {
        unimplemented!()
    }
}

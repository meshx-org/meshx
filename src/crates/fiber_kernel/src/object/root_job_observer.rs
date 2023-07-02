use std::sync::Arc;

use crate::koid;
use fiber_rust::sys;

use super::{Dispatcher, Handle, JobDispatcher, SignalObserver, TriggerMode};

#[derive(Debug)]
pub(crate) struct RootJobObserver {
    root_job: Arc<JobDispatcher>,
    koid: sys::fx_koid_t,
}

impl RootJobObserver {
    pub(crate) fn new(root_job: Arc<JobDispatcher>, root_job_handle: Arc<Handle>) -> Arc<Self> {
        let observer = Arc::new(Self {
            root_job: root_job.clone(),
            koid: koid::generate(),
        });

        root_job.add_observer(
            observer.clone(),
            root_job_handle,
            sys::FX_JOB_NO_PROCESSES | sys::FX_JOB_NO_JOBS,
            TriggerMode::Level,
        );

        observer
    }
}

impl SignalObserver for RootJobObserver {
    fn on_match(&self, signals: sys::fx_signals_t) {
        // Remember, the |root_job_|'s Dispatcher lock is held for the duration of
        // this method. Take care to avoid calling anything that might attempt to
        // acquire that lock.

        log::debug!("on match: {:x}", signals);

        // self.callback_();
    }

    fn on_cancel(&self, signals: sys::fx_signals_t) {}

    fn get_triggering_signals(&self) -> sys::fx_signals_t {
        unimplemented!()
    }

    fn set_triggeting_signals(&self, signals: sys::fx_signals_t) {
        unimplemented!()
    }

    fn set_handle(&self, handle: Arc<Handle>) {
        unimplemented!()
    }

    fn get_koid(&self) -> sys::fx_koid_t {
        self.koid
    }
}

use std::rc::Rc;

use crate::object::{HandleTable, JobDispatcher, KernelHandle, VmarDispatcher};
use fiber_sys as sys;

pub(crate) struct ProcessDispatcher {}

impl ProcessDispatcher {
    pub fn create(
        parent_job: Rc<JobDispatcher>,
        name: String,
        options: u32,
    ) -> (
        sys::fx_status_t,
        KernelHandle<ProcessDispatcher>,
        sys::fx_rights_t,
        KernelHandle<VmarDispatcher>,
        sys::fx_rights_t,
    ) {
        unimplemented!()
    }

    pub fn get_current() -> ProcessDispatcher {
        ProcessDispatcher {}
    }

    pub fn handle_table(&self) -> HandleTable {
        unimplemented!()
    }

    pub fn enforce_basic_policy(&self, policy: sys::fx_policy_t) -> sys::fx_status_t {
        sys::FX_OK
    }
}

use std::rc::Rc;

use crate::object::{HandleTable, JobDispatcher, KernelHandle, VmarDispatcher};
use fiber_sys as sys;

// state of the process
enum State {
    INITIAL, // initial state, no thread present in process
    RUNNING, // first thread has started and is running
    DYING,   // process has delivered kill signal to all threads
    DEAD,    // all threads have entered DEAD state and potentially dropped refs on process
}

pub(crate) struct ProcessDispatcher {}

// Dispatcher implementation.
impl ProcessDispatcher {
    fn get_type() -> sys::fx_obj_type_t {
        sys::FX_OBJ_TYPE_PROCESS
    }

    fn get_koid() -> sys::fx_koid_t {
        0
    }
}

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

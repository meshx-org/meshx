use std::rc::Rc;

use crate::object::{HandleTable, JobDispatcher, JobPolicy, KernelHandle, VmarDispatcher};
use crate::process_context::with_context;
use fiber_sys as sys;

// state of the process
enum State {
    INITIAL, // initial state, no thread present in process
    RUNNING, // first thread has started and is running
    DYING,   // process has delivered kill signal to all threads
    DEAD,    // all threads have entered DEAD state and potentially dropped refs on process
}

#[derive(Debug)]
pub(crate) struct ProcessDispatcher {
    handle_table: HandleTable,
    name: String,
    job: Rc<JobDispatcher>,
    policy: JobPolicy,
}

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
    pub(crate) fn new(job: Rc<JobDispatcher>, name: String, flags: u32) -> ProcessDispatcher {
        let new_process = ProcessDispatcher {
            job: job.clone(),
            policy: job.get_policy(),
            handle_table: HandleTable::new(),
            name: name.clone(),
        };

        //  kcounter_add(dispatcher_process_create_count, 1);
        new_process
    }

    pub(crate) fn create(
        parent_job: Rc<JobDispatcher>,
        name: String,
        options: u32,
    ) -> (
        sys::fx_status_t,
        KernelHandle<Rc<ProcessDispatcher>>,
        sys::fx_rights_t,
        KernelHandle<Rc<VmarDispatcher>>,
        sys::fx_rights_t,
    ) {
        unimplemented!()
    }

    pub fn get_current() -> Rc<ProcessDispatcher> {
        with_context(|context| context.process.clone())
    }

    pub fn handle_table(&self) -> HandleTable {
        self.handle_table
    }

    pub fn enforce_basic_policy(&self, policy: sys::fx_policy_t) -> sys::fx_status_t {
        sys::FX_OK
    }
}

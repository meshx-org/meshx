use fiber_sys as sys;

use std::rc::Rc;

use crate::object::{IDispatcher, INamed, KernelHandle};

// The starting max_height value of the root job.
static ROOT_JOB_MAX_HEIGHT: u32 = 32;
static ROOT_JOB_NAME: &str = "root";

#[derive(Debug, Clone, Copy)]
pub(crate) struct JobPolicy;

#[derive(Debug, PartialEq)]
enum State {
    READY,
    KILLING,
    DEAD,
}

type RawJobList = Vec<*const JobDispatcher>;

#[derive(Debug)]
pub(crate) struct JobDispatcher {
    parent_job: Option<Rc<JobDispatcher>>,
    max_height: u32,

    // The user-friendly job name. For debug purposes only. That
    // is, there is no mechanism to mint a handle to a job via this name.
    name: String,

    // The common |get_lock()| protects all members below.
    state: State,     // TA_GUARDED(get_lock());
    return_code: i64, //  TA_GUARDED(get_lock());

    // TODO(cpu): The OOM kill system is incomplete, see fxbug.dev/32577 for details.
    kill_on_oom: bool, // TA_GUARDED(get_lock());

    // Access to the pointers in these lists, especially any promotions to
    // RefPtr, must be handled very carefully, because the children can die
    // even when |lock_| is held. See ForEachChildInLocked() for more details
    // and for a safe way to enumerate them.
    jobs: RawJobList, // TA_GUARDED(get_lock());
    // RawProcessList procs_ TA_GUARDED(get_lock());
    policy: JobPolicy, // TA_GUARDED(get_lock());
}

// Dispatcher implementation.
impl IDispatcher for JobDispatcher {
    fn get_type() -> sys::fx_obj_type_t {
        sys::FX_OBJ_TYPE_JOB
    }

    fn get_koid(&self) -> sys::fx_koid_t {
        0
    }

    fn get_related_koid(&self) -> sys::fx_koid_t {
        if self.parent_job.is_some() {
            self.parent_job.as_ref().unwrap().get_koid()
        } else {
            0
        }
    }

    fn default_rights() -> sys::fx_rights_t {
        sys::FX_RIGHT_EXECUTE
    }
}


impl INamed for JobDispatcher {
    fn set_name(&self, name: String) -> sys::fx_status_t {
        sys::FX_OK
    }

    fn get_name(&self) -> String {
        self.name.clone()
    }
}

impl JobDispatcher {
    pub fn create_root_job() -> Rc<JobDispatcher> {
        unimplemented!()
    }

    pub fn create(
        parent: Rc<JobDispatcher>,
        flags: u32,
    ) -> (sys::fx_status_t, Option<KernelHandle<JobDispatcher>>, sys::fx_rights_t) {
        if parent.max_height() == 0 {
            // The parent job cannot have children.
            return (sys::FX_ERR_OUT_OF_RANGE, None, 0);
        }

        let new_handle = KernelHandle {
            dispatcher: JobDispatcher::new(flags, Some(parent.clone()), parent.get_policy()).into(),
        };

        if !parent.add_child_job(new_handle.dispatcher()) {
            return (sys::FX_ERR_OUT_OF_RANGE, None, 0);
        }

        (sys::FX_OK, Some(new_handle), JobDispatcher::default_rights())
    }

    pub(crate) fn new(flags: u32, parent: Option<Rc<JobDispatcher>>, policy: JobPolicy) -> JobDispatcher {
        JobDispatcher {
            parent_job: parent.clone(),
            max_height: if parent.is_some() {
                parent.unwrap().max_height() - 1
            } else {
                0
            },
            state: State::READY,
            return_code: 0,
            policy,
            kill_on_oom: false,
            name: String::from(""),
            jobs: vec![],
        }

        // kcounter_add(dispatcher_job_create_count, 1);
    }
}

// Job methods.
impl JobDispatcher {
    fn parent(&self) -> Option<Rc<JobDispatcher>> {
        self.parent_job.clone()
    }

    fn max_height(&self) -> u32 {
        return self.max_height;
    }

    pub(crate) fn get_policy(&self) -> JobPolicy {
        // Guard<Mutex> guard{ get_lock() };
        self.policy
    }

    pub(crate) fn add_child_job(&self, job: &Rc<JobDispatcher>) -> bool {
        //canary_.Assert();
        //Guard<Mutex> guard{get_lock()};

        if self.state != State::READY {
            return false;
        }

        // Put the new job after our next-youngest child, or us if we have none.
        //
        // We try to make older jobs closer to the root (both hierarchically and
        // temporally) show up earlier in enumeration.
        // TODO: JobDispatcher* neighbor = if self.jobs_.is_empty() { this } else { &self.jobs_.back() };

        // This can only be called once, the job should not already be part
        // of any job tree.
        // DEBUG_ASSERT(!fbl::InContainer<JobDispatcher::RawListTag>(*job));
        // DEBUG_ASSERT(neighbor != job.get());

        // self.jobs.push(job.as_ref());

        todo!();

        // TODO: UpdateSignalsLocked();
        return true;
    }
}

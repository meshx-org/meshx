use fiber_sys as sys;

use std::any::Any;
use std::rc::Rc;
use std::sync::{Arc, RwLock};

use super::{GenericDispatcher, Handle};
use crate::object::{BaseDispatcher, Dispatcher, INamed, KernelHandle, ProcessDispatcher, TypedDispatcher};

// The starting max_height value of the root job.
static ROOT_JOB_MAX_HEIGHT: u32 = 32;
static ROOT_JOB_NAME: &str = "root";

#[derive(Debug, Clone, Copy)]
pub(crate) struct JobPolicy;

impl JobPolicy {
    fn create_root_policy() -> Self {
        Self {}
    }
}

#[derive(Debug, PartialEq)]
enum State {
    READY,
    KILLING,
    DEAD,
}

#[derive(Debug)]
struct GuardedState {
    // Access to the pointers in these lists, especially any promotions to
    // RefPtr, must be handled very carefully, because the children can die
    // even when |lock_| is held. See ForEachChildInLocked() for more details
    // and for a safe way to enumerate them.
    jobs: Vec<Arc<JobDispatcher>>,      // TA_GUARDED(get_lock());
    procs: Vec<Arc<ProcessDispatcher>>, // TA_GUARDED(get_lock());
}

trait S: Send + Sync {}
impl S for GuardedState {}

#[derive(Debug)]
pub(crate) struct JobDispatcher {
    base: BaseDispatcher,

    parent_job: Option<Arc<JobDispatcher>>,
    max_height: u32,

    // The user-friendly job name. For debug purposes only. That
    // is, there is no mechanism to mint a handle to a job via this name.
    name: String,

    // The common |get_lock()| protects all members below.
    state: State,     // TA_GUARDED(get_lock());
    return_code: i64, //  TA_GUARDED(get_lock());

    // TODO(cpu): The OOM kill system is incomplete, see fxbug.dev/32577 for details.
    kill_on_oom: bool, // TA_GUARDED(get_lock());

    policy: JobPolicy, // TA_GUARDED(get_lock());

    guarded: RwLock<GuardedState>,
}

// Dispatcher implementation.
impl Dispatcher for JobDispatcher {
    fn get_koid(&self) -> sys::fx_koid_t {
        self.base.get_koid()
    }

    fn get_related_koid(&self) -> sys::fx_koid_t {
        if self.parent_job.is_some() {
            self.parent_job.as_ref().unwrap().get_koid()
        } else {
            0
        }
    }

    fn base(&self) -> &BaseDispatcher {
        &self.base
    }
}

impl TypedDispatcher for JobDispatcher {
    fn default_rights() -> sys::fx_rights_t {
        sys::FX_RIGHT_NONE
    }

    fn get_type() -> sys::fx_obj_type_t {
        sys::FX_OBJ_TYPE_JOB
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
    pub fn create_root_job() -> Arc<JobDispatcher> {
        let job = JobDispatcher::new(0, None, JobPolicy::create_root_policy());
        job.set_name("root".to_string());
        job
    }

    pub fn create(
        parent: Arc<JobDispatcher>,
        flags: u32,
    ) -> (sys::fx_status_t, Option<KernelHandle<JobDispatcher>>, sys::fx_rights_t) {
        if parent.max_height() == 0 {
            // The parent job cannot have children.
            return (sys::FX_ERR_OUT_OF_RANGE, None, 0);
        }

        let new_handle = KernelHandle::new(GenericDispatcher::JobDispatcher(
            JobDispatcher::new(flags, Some(parent.clone()), parent.get_policy()).into(),
        ));

        // extract the dispatcher from the handle
        let child_job = new_handle.dispatcher();

        let child_job = match child_job {
            GenericDispatcher::JobDispatcher(job) => job,
            _ => panic!(),
        };

        if !parent.add_child_job(&child_job) {
            return (sys::FX_ERR_OUT_OF_RANGE, None, 0);
        }

        (sys::FX_OK, Some(new_handle), JobDispatcher::default_rights())
    }

    pub(crate) fn new(flags: u32, parent: Option<Arc<JobDispatcher>>, policy: JobPolicy) -> Arc<JobDispatcher> {
        log::debug!("JobDispatcher::new( {:?}, {:?}, {:?})", flags, parent, policy);

        let job = JobDispatcher {
            base: BaseDispatcher::new(0),
            parent_job: parent.clone(),
            max_height: if parent.is_some() {
                parent.unwrap().max_height() - 1
            } else {
                ROOT_JOB_MAX_HEIGHT
            },
            name: String::from(""),
            state: State::READY,
            return_code: 0,
            kill_on_oom: false,
            policy,
            guarded: RwLock::new(GuardedState {
                jobs: vec![],
                procs: vec![],
            }),
        };

        Arc::new(job)
        // kcounter_add(dispatcher_job_create_count, 1);
    }
}

// Job methods.
impl JobDispatcher {
    fn parent(&self) -> Option<Arc<JobDispatcher>> {
        self.parent_job.clone()
    }

    fn max_height(&self) -> u32 {
        return self.max_height;
    }

    pub(crate) fn get_policy(&self) -> JobPolicy {
        // Guard<Mutex> guard{ get_lock() };
        self.policy
    }

    pub(crate) fn add_child_job(&self, job: &Arc<JobDispatcher>) -> bool {
        //canary_.Assert();
        //Guard<Mutex> guard{get_lock()};
        let mut guarded_state = self.guarded.write().unwrap();

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

        guarded_state.jobs.push(job.clone());

        // TODO: UpdateSignalsLocked();
        return true;
    }

    pub(crate) fn add_child_process(&self, process: &Arc<ProcessDispatcher>) -> bool {
        //canary_.Assert();
        let mut guarded_state = self.guarded.write().unwrap();

        if self.state != State::READY {
            return false;
        }

        guarded_state.procs.push(process.clone());

        // TODO:  UpdateSignalsLocked();
        return true;
    }
}

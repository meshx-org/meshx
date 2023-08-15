use std::any::Any;
use std::future::IntoFuture;
use std::rc::Rc;
use std::sync::Arc;

use crate::object::{
    BaseDispatcher, Dispatcher, HandleTable, JobDispatcher, JobPolicy, KernelHandle, TypedDispatcher, VMODispatcher,
};
use crate::process_context::{self, get_last_process, Context, ScopeGuard};
use fiber_sys as sys;

use super::GenericDispatcher;

use switcheroo::Generator;
use switcheroo::{stack::*, Yielder};

// state of the process
#[derive(Debug, PartialEq)]
enum State {
    INITIAL, // initial state, no thread present in process
    RUNNING, // first thread has started and is running
    DYING,   // process has delivered kill signal to all threads
    DEAD,    // all threads have entered DEAD state and potentially dropped refs on process
}

#[derive(Debug)]
pub(crate) struct ProcessDispatcher {
    base: BaseDispatcher,
    handle_table: Option<HandleTable>,
    name: String,
    job: Arc<JobDispatcher>,
    policy: JobPolicy,
    state: State,
}

// Dispatcher implementation.
impl Dispatcher for ProcessDispatcher {
    fn get_koid(&self) -> sys::fx_koid_t {
        self.base.get_koid()
    }

    fn get_related_koid(&self) -> sys::fx_koid_t {
        0
    }

    fn base(&self) -> &super::BaseDispatcher {
        &self.base
    }
}

impl TypedDispatcher for ProcessDispatcher {
    fn default_rights() -> sys::fx_rights_t {
        sys::FX_DEFAULT_PROCESS_RIGHTS
    }

    fn get_type() -> sys::fx_obj_type_t {
        sys::FX_OBJ_TYPE_PROCESS
    }
}

impl ProcessDispatcher {
    fn new(job: Arc<JobDispatcher>, name: String, flags: u32) -> Arc<ProcessDispatcher> {
        log::debug!("ProcessDispatcher::new({:?})", name);

        let mut new_process = ProcessDispatcher {
            base: BaseDispatcher::new(0),
            job: job.clone(),
            policy: job.get_policy(),
            handle_table: None,
            name: name.clone(),
            state: State::INITIAL,
        };

        let handle_table = HandleTable::new(&new_process);
        new_process.handle_table = Some(handle_table);

        //  kcounter_add(dispatcher_process_create_count, 1);
        Arc::from(new_process)
    }

    pub(crate) fn create(
        parent_job: Arc<JobDispatcher>,
        name: String,
        flags: u32,
    ) -> Result<(KernelHandle<ProcessDispatcher>, sys::fx_rights_t), sys::fx_status_t> {
        let handle = KernelHandle::new(GenericDispatcher::ProcessDispatcher(ProcessDispatcher::new(
            parent_job.clone(),
            name.clone(),
            flags,
        )));

        let process_dispatcher = match handle.dispatcher() {
            GenericDispatcher::ProcessDispatcher(handle) => handle,
            _ => panic!("Wrong dispatcher type"),
        };

        let status = process_dispatcher.init();
        if status != sys::FX_OK {
            return Err(status);
        }

        let child_process = handle.dispatcher().as_process_dispatcher().unwrap();

        // Only now that the process has been fully created and initialized can we register it with its
        // parent job. We don't want anyone to see it in a partially initalized state.
        if !parent_job.add_child_process(&child_process) {
            return Err(sys::FX_ERR_BAD_STATE);
        }

        Ok((handle, ProcessDispatcher::default_rights()))
    }

    fn init(&self) -> sys::fx_status_t {
        //Guard<Mutex> guard{get_lock()};
        debug_assert!(self.state == State::INITIAL);

        // create an address space for this process, named after the process's koid.
        //let aspace_name: [u8; ZX_MAX_NAME_LEN] = format!("proc:{}", self.get_koid()).into();

        //let aspace_ = VmAspace::Create(VmAspace::TYPE_USER, aspace_name);

        //if (!aspace_) {
        //  trace!("error creating address space\n");
        //  return sys::FX_ERR_NO_MEMORY;
        //}

        return sys::FX_OK;
    }

    // Start this process running with the provided entry state, only
    // valid to be called on a thread in the INITIALIZED state that has not yet been started. If
    // `ensure_initial_thread` is true, the thread will only start if it is the first thread in the
    // process.
    pub(crate) fn start(
        self: Arc<Self>,
        entry: fn(arg1: sys::fx_handle_t, arg2: sys::fx_handle_t),
        arg1: sys::fx_handle_t,
        arg2: sys::fx_handle_t,
    ) {
        log::debug!("ProcessDispatcher::start({:?}, {:?})", entry, self.name);

        let stack = OneMbStack::new().unwrap();

        let mut task = Generator::<_, _, OneMbStack>::new(stack, |yielder, input: u32| {
            let context = Context {
                process: self,
                yielder: yielder as *const Yielder<u32, u32>,
            };

            // Make sure to save the guard, see documentation for more information
            let _guard = ScopeGuard::new(context);

            entry(arg1, input);
        });

        log::debug!("{:?}", task.resume(0));
        log::debug!("{:?}", task.resume(1));
        //log::debug!("{:?}", task.resume(1));
    }

    pub(crate) fn get_current() -> Context {
        log::trace!("ProcessDispatcher::get_current()");
        get_last_process()
    }

    pub(crate) fn handle_table(&self) -> &HandleTable {
        self.handle_table.as_ref().unwrap()
    }

    pub(crate) fn enforce_basic_policy(&self, policy: sys::fx_policy_t) -> sys::fx_status_t {
        sys::FX_OK
    }
}

/*impl ThreadDispatcher {
    // low level entry point for the fiber
    fn start_fiber(arg: *const ()) -> i32 {
        LTRACE_ENTRY;
        let t: *const ThreadDispatcher = arg as *const ThreadDispatcher;
        // IWBN to dump the values just before calling |arch_enter_uspace()|
        // but at that point they're in |iframe| and may have been modified by
        // the debugger user, and fetching them out of the iframe will require
        // architecture-specific code. Instead just print them here. This is just
        // for tracing which is generally off, and then only time the values will
        // have changed is if a debugger user changes them. KISS.
        trace!("arch_enter_uspace SP: %# PC: %#, ARG1: %#, ARG2: %#\n", t.user_entry_.sp, t.user_entry_.pc, t.user_entry_.arg1, t.user_entry_.arg2);

        // Initialize an iframe for entry into userspace.
        // We need all registers accessible from the ZX_EXCP_THREAD_STARTING
        // exception handler (the debugger wants the thread to look as if the
        // thread is at the first instruction). For architectural exceptions the
        // general regs are left in the iframe for speed and simplicity. To keep
        // things simple we use the same scheme.
        let iframe = iframe_t{};
        arch_setup_uspace_iframe(&iframe, t.user_entry.pc, t.user_entry.sp, t.user_entry.arg1);
        let context = arch_exception_context_t {};
        context.frame = &iframe;

        // Notify job debugger if attached.
        if (t.is_initial_thread_) {
          t.process_.OnProcessStartForJobDebugger(t, &context);
        }

        // Notify debugger if attached.
        t.HandleSingleShotException(t.process.exceptionate(Exceptionate::Type::kDebug), ZX_EXCP_THREAD_STARTING, context);

        arch_iframe_process_pending_signals(&iframe);
        // switch to user mode and start the process
        arch_enter_uspace(&iframe);
        unreachable!();
      }
}*/

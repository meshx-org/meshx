#![feature(trait_upcasting)]
#![feature(local_key_cell_methods)]

// Copyright 2023 MeshX Contributors. All rights reserved.
pub mod koid;
// pub mod userboot;

mod object;
mod process_context;

use object::{HandleOwner, PortDispatcher};
use std::{
    cell::Cell,
    fmt,
    sync::{Arc, Mutex},
};
use tracing::{event, instrument, Level};

use fiber_sys as sys;

use crate::object::{
    Dispatcher, GenericDispatcher, Handle, JobDispatcher, JobPolicy, KernelHandle, ProcessDispatcher, RootJobObserver,
    TypedDispatcher,
};

pub struct Kernel {
    cb: fn(&object::ProcessObject),
    boot_process: Option<Arc<dyn Fn() + Send + Sync>>,

    // All jobs and processes of this Kernel are rooted at this job.
    root_job: Option<Arc<JobDispatcher>>,
    root_job_handle: Option<HandleOwner>,

    // Watch the root job, taking action (such as a system reboot) if it ends up
    // with no children.
    root_job_observer: Mutex<Option<Arc<RootJobObserver>>>,
}

impl fmt::Debug for Kernel {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("Kernel").finish()
    }
}

impl fiber_sys::System for Kernel {
    fn sys_handle_close(&self, handle: sys::fx_handle_t) -> sys::fx_status_t {
        0
    }

    fn sys_handle_duplicate(
        &self,
        handle: sys::fx_handle_t,
        rights: sys::fx_rights_t,
        out: *const sys::fx_handle_t,
    ) -> sys::fx_status_t {
        0
    }

    fn sys_handle_replace(
        &self,
        handle: sys::fx_handle_t,
        rights: sys::fx_rights_t,
        out: *const sys::fx_handle_t,
    ) -> sys::fx_status_t {
        0
    }

    fn sys_object_get_info(
        &self,
        handle: sys::fx_handle_t,
        topic: u32,
        buffer: *const u8,
        buffer_size: usize,
    ) -> sys::fx_status_t {
        0
    }

    fn sys_process_create(
        &self,
        job_handle: sys::fx_handle_t,
        name: *const u8,
        mut name_size: usize,
        options: u32,
        proc_handle: *mut sys::fx_handle_t,
        vmar_handle: *mut sys::fx_handle_t,
    ) -> sys::fx_status_t {
        // currently, the only valid option value is 0
        if options != 0 {
            return sys::FX_ERR_INVALID_ARGS;
        }

        let current = ProcessDispatcher::get_current();
        let up = current.process.clone();

        // We check the policy against the process calling zx_process_create, which
        // is the operative policy, rather than against |job_handle|. Access to
        // |job_handle| is controlled by the rights associated with the handle.
        let status: sys::fx_status_t = up.enforce_basic_policy(sys::FX_POLICY_NEW_PROCESS);
        if status != sys::FX_OK {
            return status;
        }

        // Silently truncate the given name.
        if name_size > sys::FX_MAX_NAME_LEN {
            name_size = sys::FX_MAX_NAME_LEN;
        }
        log::trace!("name_size = {}", name_size);

        // TODO: remove unsafe opertaion
        let name = unsafe { String::from_raw_parts(name as *mut u8, name_size, sys::FX_MAX_NAME_LEN) };

        log::trace!("name = {}", name.clone());

        let result =
            up.handle_table()
                .get_dispatcher_with_rights(up.as_ref(), job_handle, sys::FX_RIGHT_MANAGE_PROCESS);

        if let Err(status) = result {
            return status;
        }

        let parent_job = result.unwrap().as_job_dispatcher().unwrap();

        // create a new process dispatcher
        let result = ProcessDispatcher::create(parent_job, name, options);

        if let Err(status) = result {
            return status;
        }

        let (new_process_handle, process_rights) = result.expect("should be a valid value");

        // TODO: let koid: u32 = new_process_handle.dispatcher().get_koid();
        // TODO: ktrace(TAG_PROC_CREATE, koid, 0, 0, 0);
        // TODO: ktrace_name(TAG_PROC_NAME, koid, 0, buf);
        // Give arch-specific tracing a chance to record process creation.
        // TODO: arch_trace_process_create( koid, new_vmar_handle.dispatcher().vmar().aspace().arch_aspace().arch_table_phys());

        let handle = Handle::make(new_process_handle, process_rights);

        status
    }

    fn sys_process_start(
        &self,
        handle: sys::fx_handle_t,
        entry: sys::fx_vaddr_t,
        arg1: sys::fx_handle_t,
    ) -> sys::fx_status_t {
        let current = ProcessDispatcher::get_current();
        let up = current.process.clone();

        let result = up.handle_table().get_dispatcher_with_rights(up.as_ref(), handle, 0);

        if let Err(status) = result {
            return status;
        }

        let process = result.unwrap();

        // (self.cb)(&ProcessObject(KernelObject));

        0
    }

    fn sys_process_exit(&self, retcode: i64) -> sys::fx_status_t {
        0
    }

    #[instrument(skip(self))]
    fn sys_job_create(
        &self,
        parent_job: sys::fx_handle_t,
        options: u32,
        out: *const sys::fx_handle_t,
    ) -> sys::fx_status_t {
        if options != 0 {
            return sys::FX_ERR_INVALID_ARGS;
        }

        let current = ProcessDispatcher::get_current();
        let up = current.process.clone();

        let result = up
            .handle_table()
            .get_dispatcher_with_rights(up.as_ref(), parent_job, sys::FX_RIGHT_MANAGE_JOB);

        if let Err(status) = result {
            return status;
        }

        let parent_job = result.unwrap().as_job_dispatcher().unwrap();

        let (status, handle, rights) = JobDispatcher::create(parent_job, options);

        if status == sys::FX_OK && handle.is_some() {
            let h_ = Handle::make(handle.expect(""), rights);
        }

        status
    }

    fn sys_job_set_critical(&self, job: sys::fx_handle_t, options: u32, process: sys::fx_handle_t) -> sys::fx_status_t {
        0
    }

    fn sys_job_set_policy(
        &self,
        handle: sys::fx_handle_t,
        options: u32,
        topic: u32,
        policy: *const u8,
        policy_size: u32,
    ) -> sys::fx_status_t {
        0
    }

    fn sys_task_kill(&self, handle: sys::fx_handle_t) -> sys::fx_status_t {
        0
    }

    fn sys_object_signal_peer(&self, handle: sys::fx_handle_t, clear_mask: u32, set_mask: u32) -> sys::fx_status_t {
        todo!()
    }

    fn sys_object_signal(&self, handle: sys::fx_handle_t, clear_mask: u32, set_mask: u32) -> sys::fx_status_t {
        todo!()
    }

    fn sys_object_wait_one(
        &self,
        handle: sys::fx_handle_t,
        waitfor: sys::fx_signals_t,
        deadline: sys::fx_time_t,
        observed: *mut sys::fx_signals_t,
    ) -> sys::fx_status_t {
        todo!()
    }

    fn sys_object_wait_async(
        &self,
        handle_value: sys::fx_handle_t,
        port_handle_value: sys::fx_handle_t,
        key: u64,
        signals: sys::fx_signals_t,
        options: u32,
    ) -> sys::fx_status_t {
        if (options & !(sys::FX_WAIT_ASYNC_TIMESTAMP | sys::FX_WAIT_ASYNC_EDGE)) != 0 {
            return sys::FX_ERR_INVALID_ARGS;
        }

        let current = ProcessDispatcher::get_current();
        let up = current.process.clone();

        let observer = {
            // let guard = { up.handle_table().get_lock() };

            // Note, we're doing this all while holding the handle table lock for two reasons.
            //
            // First, this thread may be racing with another thread that's closing the last handle to
            // the port. By holding the lock we can ensure that this syscall behaves as if the port was
            // closed just *before* the syscall started or closed just *after* it has completed.
            //
            // Second, MakeObserver takes a Handle. By holding the lock we ensure the Handle isn't
            // destroyed out from under it.

            let port_handle = up.handle_table().get_handle_locked(up.as_ref(), port_handle_value);
            if port_handle.is_none() {
                return sys::FX_ERR_BAD_HANDLE;
            }

            let disp = port_handle.unwrap().dispatcher().as_port_dispatcher();
            if disp.is_none() {
                return sys::FX_ERR_WRONG_TYPE;
            }

            let port = disp.unwrap();

            // TODO: we should check for rights here
            //  if !port_handle.has_rights(sys::FX_RIGHT_WRITE) {
            //      return sys::FX_ERR_ACCESS_DENIED;
            //  }

            let handle = up.handle_table().get_handle_locked(up.as_ref(), handle_value);

            if handle.is_none() {
                return sys::FX_ERR_BAD_HANDLE;
            }

            // TODO: we should check for rights here
            // if !handle.has_rights(sys::FX_RIGHT_WAIT) {
            //     return sys::FX_ERR_ACCESS_DENIED;
            // }

            PortDispatcher::make_observer(port, options, handle.unwrap(), key, signals)
        };

        // TODO spawn async task here that waits until signal recieved.
        // tokio::task::spawn(observer)

        sys::FX_OK
    }

    fn sys_channel_create(
        &self,
        options: u32,
        out0: *mut sys::fx_handle_t,
        out1: *mut sys::fx_handle_t,
    ) -> sys::fx_status_t {
        todo!()
    }

    #[instrument(target = "klog", skip(self, handles, bytes))]
    fn sys_channel_read(
        &self,
        handle: sys::fx_handle_t,
        options: u32,
        bytes: *mut u8,
        handles: *mut sys::fx_handle_t,
        num_bytes: u32,
        num_handles: u32,
        actual_bytes: *mut u32,
        actual_handles: *mut u32,
    ) -> sys::fx_status_t {
        let up = ProcessDispatcher::get_current();

        sys::FX_OK
    }

    #[tracing::instrument]
    fn sys_channel_read_etc(
        &self,
        handle: sys::fx_handle_t,
        options: u32,
        bytes: *mut u8,
        handles: *mut sys::fx_handle_info_t,
        num_bytes: u32,
        num_handles: u32,
        actual_bytes: *mut u32,
        actual_handles: *mut u32,
    ) -> sys::fx_status_t {
        todo!()
    }

    fn sys_channel_write(
        &self,
        handle: sys::fx_handle_t,
        options: u32,
        bytes: *const u8,
        num_bytes: u32,
        handles: *const sys::fx_handle_t,
        num_handles: u32,
    ) -> sys::fx_status_t {
        todo!()
    }

    fn sys_channel_write_etc(
        &self,
        handle: sys::fx_handle_t,
        options: u32,
        bytes: *const u8,
        num_bytes: u32,
        handles: *const sys::fx_handle_disposition_t,
        num_handles: u32,
    ) -> sys::fx_status_t {
        todo!()
    }

    fn sys_channel_call_etc(
        &self,
        handle: sys::fx_handle_t,
        options: u32,
        deadline: sys::fx_time_t,
        args: *const sys::fx_channel_call_etc_args_t,
        actual_bytes: *const u32,
        actual_handles: *const u32,
    ) -> sys::fx_status_t {
        todo!()
    }

    fn sys_vmo_create(&self, size: u64, options: u32, out: *mut sys::fx_handle_t) -> sys::fx_status_t {
        todo!()
    }

    fn sys_vmo_read(
        &self,
        handle: sys::fx_handle_t,
        buffer: *mut u8,
        offset: u64,
        buffer_size: usize,
    ) -> sys::fx_status_t {
        todo!()
    }

    fn sys_vmo_write(
        &self,
        handle: sys::fx_handle_t,
        buffer: *const u8,
        offset: u64,
        buffer_size: usize,
    ) -> sys::fx_status_t {
        todo!()
    }

    fn sys_vmo_get_size(&self, handle: sys::fx_handle_t, size: *mut u64) -> sys::fx_status_t {
        todo!()
    }

    fn sys_port_create(&self, options: u32, out: *mut sys::fx_handle_t) -> sys::fx_status_t {
        todo!()
    }

    fn sys_port_queue(&self, handle: sys::fx_handle_t, packet: *const sys::fx_port_packet_t) -> sys::fx_status_t {
        todo!()
    }

    fn sys_port_wait(
        &self,
        handle: sys::fx_handle_t,
        deadline: sys::fx_time_t,
        packet: *mut sys::fx_port_packet_t,
    ) -> sys::fx_status_t {
        todo!()
    }

    fn sys_port_cancel(&self, handle: sys::fx_handle_t, source: sys::fx_handle_t, key: u64) -> sys::fx_status_t {
        todo!()
    }
}

type OnProcessStartHook = fn(&object::ProcessObject);

use std::{
    future::Future,
    pin::Pin,
    task::{Context, Poll},
};

impl Kernel {
    pub fn new(on_process_start_cb: OnProcessStartHook) -> Self {
        Self {
            cb: on_process_start_cb,
            boot_process: None,

            root_job: None,
            root_job_handle: None,
            root_job_observer: Mutex::new(None),
        }
    }

    pub fn init(&mut self) {
        // Create root job.
        let root_job = JobDispatcher::create_root_job();
        self.root_job = Some(root_job.clone());

        //if constexpr (KERNEL_BASED_MEMORY_ATTRIBUTION) {
        //    // Insert the kernel's attribution object as a child of the root job.
        //    fbl::RefPtr<AttributionObject> kernel = AttributionObject::GetKernelAttribution();
        //    kernel->AddToGlobalListWithKoid(root_job_->attribution_objects_end(), ZX_KOID_INVALID);
        //}

        // Create handle.
        self.root_job_handle = Some(Handle::make::<JobDispatcher>(
            KernelHandle::new(GenericDispatcher::JobDispatcher(root_job)),
            JobDispatcher::default_rights(),
        ));

        assert!(self.root_job_handle.is_some());
    }

    pub fn start(&self) {
        let rt = tokio::runtime::Builder::new_multi_thread()
            .enable_all()
            .build()
            .unwrap();

        rt.block_on(async {
            // userboot::userboot_init(self);

            log::info!("Now wait until the root job is childless.");

            println!("Hello world");
        })
    }

    // Returns the job that is the ancestor of all other tasks.
    pub(crate) fn get_root_job_dispatcher(&self) -> Arc<JobDispatcher> {
        debug_assert!(self.root_job.is_some());

        self.root_job.as_ref().unwrap().clone()
    }

    pub(crate) fn get_root_job_handle(&self) -> HandleOwner {
        self.root_job_handle.clone().unwrap()
    }

    /// Start the RootJobObserver. Must be called after the root job has at
    /// least one child process or child job.
    pub(crate) fn start_root_job_observer(&self) {
        let mut locked = self.root_job_observer.lock().unwrap();

        assert!(locked.is_none());
        debug_assert!(self.root_job.is_some());

        let observer = RootJobObserver::new(self.root_job.clone().unwrap(), self.root_job_handle.clone().unwrap());
        *locked = Some(observer);

        //if !ac.check() {
        //    panic!("root-job: failed to allocate observer\n");
        //}

        // Initialize the memory watchdog.
        // self.memory_watchdog_.Init(this);
    }

    pub fn register_boot_process<F>(&mut self, f: F)
    where
        F: Fn() + Send + Sync + 'static,
    {
        let process = ProcessDispatcher::create(JobDispatcher::new(0, None, JobPolicy), String::from(""), 0)
            .unwrap()
            .0
            .dispatcher();

        self.boot_process = Some(Arc::new(f));
    }
}

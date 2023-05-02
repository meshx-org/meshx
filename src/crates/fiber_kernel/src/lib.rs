#![feature(trait_upcasting)]

// Copyright 2023 MeshX Contributors. All rights reserved.
pub mod koid;

mod userboot;
mod object;
mod process_context;

use log::trace;
use std::rc::Rc;
use std::{fmt, sync::Arc};

use fiber_sys as sys;

use crate::object::{Handle, JobDispatcher, JobPolicy, ProcessDispatcher};

pub struct Kernel {
    cb: fn(&object::ProcessObject),
    boot_process: Option<Arc<dyn Fn() + Send + Sync>>,
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
        trace!("job handle = {}, options = {:?}\n", job_handle, options);

        // currently, the only valid option value is 0
        if options != 0 {
            return sys::FX_ERR_INVALID_ARGS;
        }

        let up = ProcessDispatcher::get_current();

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
        trace!("name_size = {}", name_size);

        // NOTE: highly unsafe opertaion
        let name = unsafe { String::from_raw_parts(name as *mut u8, name_size, sys::FX_MAX_NAME_LEN) };

        trace!("name = {}", name.clone());

        let result = up
            .handle_table()
            .get_dispatcher_with_rights(job_handle, sys::FX_RIGHT_MANAGE_PROCESS);

        if let Err(status) = result {
            return status;
        }

        let parent_job = result.unwrap();

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
        let up = ProcessDispatcher::get_current();

        let result = up.handle_table().get_dispatcher_with_rights(handle, 0);

        if let Err(status) = result {
            return status;
        }

        let process: Rc<ProcessDispatcher> = result.unwrap();

        // (self.cb)(&ProcessObject(KernelObject));

        0
    }

    fn sys_process_exit(&self, retcode: i64) -> sys::fx_status_t {
        0
    }

    fn sys_job_create(
        &self,
        parent_job: sys::fx_handle_t,
        options: u32,
        out: *const sys::fx_handle_t,
    ) -> sys::fx_status_t {
        trace!("klog sys_job_create: {}", parent_job);

        if options != 0 {
            return sys::FX_ERR_INVALID_ARGS;
        }

        let up = ProcessDispatcher::get_current();

        trace!("klog up: {:?}", up);

        let result = up
            .handle_table()
            .get_dispatcher_with_rights(parent_job, sys::FX_RIGHT_MANAGE_JOB);

        if let Err(status) = result {
            return status;
        }

        let parent_job = result.unwrap();

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
        handle: sys::fx_handle_t,
        port_handle: sys::fx_handle_t,
        key: u64,
        signals: sys::fx_signals_t,
        options: u32,
    ) -> sys::fx_status_t {
        todo!()
    }

    fn sys_channel_create(
        &self,
        options: u32,
        out0: *mut sys::fx_handle_t,
        out1: *mut sys::fx_handle_t,
    ) -> sys::fx_status_t {
        todo!()
    }

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
        todo!()
    }

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

#[inline]
pub(crate) fn process_scope<F: Send + Sync + 'static>(f: F, process: Rc<ProcessDispatcher>)
where
    F: Fn(),
{
    trace!("enter process scope");

    // Make sure to save the guard, see documentation for more information
    let _guard = process_context::ScopeGuard::new(process_context::Context { process });

    f();
}

type OnProcessStartHook = fn(&object::ProcessObject);



impl Kernel {
    pub fn new(on_process_start_cb: OnProcessStartHook) -> Self {
        Self {
            cb: on_process_start_cb,
            boot_process: None,
        }
    }

    pub fn register_boot_process<F>(&mut self, f: F)
    where
        F: Fn() + Send + Sync + 'static,
    {
        let process = ProcessDispatcher::create(Rc::from(JobDispatcher::new(0, None, JobPolicy)), String::from(""), 0)
            .unwrap()
            .0
            .dispatcher()
            .clone();

        self.boot_process = Some(Arc::new(f));
    }

    pub fn run_root(&self) {
        let refer = self.boot_process.clone();

        // TODO: do not create a process here, but use the current process
        let process = ProcessDispatcher::create(Rc::from(JobDispatcher::new(0, None, JobPolicy)), String::from(""), 0)
            .unwrap()
            .0
            .dispatcher()
            .clone();

        process_scope(
            move || {
                let process = refer.as_ref().unwrap();
                process()
            },
            process,
        );
    }
}

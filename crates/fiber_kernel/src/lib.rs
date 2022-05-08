// Copyright 2022 MeshX Contributors. All rights reserved.

mod koid;
mod object;
mod process_context;

use log::{info, trace};
use std::rc::Rc;

use fiber_sys as sys;

use crate::object::{Handle, JobDispatcher, JobPolicy, ProcessDispatcher};

pub struct Kernel {
    cb: fn(&object::ProcessObject),
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

        let (new_process_handle, process_rights, new_root_dv_handle, root_dv_rights) =
            result.expect("should be a valid value");

        // TODO: let koid: u32 = new_process_handle.dispatcher().get_koid();
        // TODO: ktrace(TAG_PROC_CREATE, koid, 0, 0, 0);
        // TODO: ktrace_name(TAG_PROC_NAME, koid, 0, buf);
        // Give arch-specific tracing a chance to record process creation.
        // TODO: arch_trace_process_create( koid, new_vmar_handle.dispatcher().vmar().aspace().arch_aspace().arch_table_phys());

        let handle = Handle::make(new_process_handle, process_rights);

        if status == sys::FX_OK {
            let handle = Handle::make(new_root_dv_handle, root_dv_rights);
        }

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
        trace!("parent = {}", parent_job);

        if options != 0 {
            return sys::FX_ERR_INVALID_ARGS;
        }

        let up = ProcessDispatcher::get_current();

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
}

#[inline]
pub fn process_scope<F, R>(f: F)
where
    F: FnOnce() -> R,
{
    // Make sure to save the guard, see documentation for more information
    let _guard = process_context::ScopeGuard::new(process_context::Context {
        process: ProcessDispatcher::new(Rc::from(JobDispatcher::new(0, None, JobPolicy)), String::from(""), 0),
    });

    f();
}

impl Kernel {
    pub fn new(on_process_start_cb: fn(&object::ProcessObject)) -> Self {
        Self {
            cb: on_process_start_cb,
        }
    }

    pub fn init(&self) {
        info!("initializing kernel");
    }

    pub fn run_root(&self) {
        // TODO
        //let res: io::Result<()> = smol::block_on(async {
        //    let mut stream = net::TcpStream::connect("example.com:80").await?;
        //    let req = b"GET / HTTP/1.1\r\nHost: example.com\r\nConnection: close\r\n\r\n";
        //    stream.write_all(req).await?;

        //    let mut stdout = Unblock::new(std::io::stdout());
        //    io::copy(stream, &mut stdout).await?;
        //    Ok(())
        //});
    }
}

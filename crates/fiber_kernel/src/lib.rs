// Copyright 2022 MeshX Contributors. All rights reserved.

mod context;
mod koid;
mod object;

use log::{debug, info, trace};
use object::{HandleOwner, KernelHandle};
use std::rc::Rc;
use std::str::FromStr;

use fiber_sys as sys;

use crate::object::{Handle, JobDispatcher, ProcessDispatcher, VmarDispatcher};

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
        trace!("job handle {}, options {:?}\n", job_handle, options);

        // currently, the only valid option value is 0
        if options != 0 {
            return sys::FX_ERR_INVALID_ARGS;
        }

        let up = ProcessDispatcher::get_current();

        // We check the policy against the process calling zx_process_create, which
        // is the operative policy, rather than against |job_handle|. Access to
        // |job_handle| is controlled by the rights associated with the handle.
        let mut result: sys::fx_status_t = up.enforce_basic_policy(sys::FX_POLICY_NEW_PROCESS);
        if result != sys::FX_OK {
            return result;
        }

        // Silently truncate the given name.
        if name_size > sys::FX_MAX_NAME_LEN {
            name_size = sys::FX_MAX_NAME_LEN;
        }
        let sp = unsafe { String::from_raw_parts(name as *mut u8, name_size, sys::FX_MAX_NAME_LEN) };

        trace!("name = {}", sp);

      

        let (result, parent_job) = up
            .handle_table()
            .get_dispatcher_with_rights(job_handle, sys::FX_RIGHT_MANAGE_PROCESS);

        // create a new process dispatcher
        let (mut result, new_process_handle, process_rights, new_root_vmar_handle, root_vmar_rights) =
            ProcessDispatcher::create(parent_job, sp, options);

        if result != sys::FX_OK {
            return result;
        }

        // TODO: let koid: u32 = new_process_handle.dispatcher().get_koid();
        // TODO: ktrace(TAG_PROC_CREATE, koid, 0, 0, 0);
        // TODO: ktrace_name(TAG_PROC_NAME, koid, 0, buf);
        // Give arch-specific tracing a chance to record process creation.
        // TODO: arch_trace_process_create( koid, new_vmar_handle.dispatcher().vmar().aspace().arch_aspace().arch_table_phys());

        let h_: HandleOwner<ProcessDispatcher> = Handle::make(new_process_handle, process_rights);
        result = sys::FX_OK;

        if result == sys::FX_OK {
            let h_: HandleOwner<VmarDispatcher> = Handle::make(new_root_vmar_handle, root_vmar_rights);
            result = sys::FX_OK;
        }

        return result;
    }

    fn sys_process_start(
        &self,
        handle: sys::fx_handle_t,
        entry: sys::fx_vaddr_t,
        arg1: sys::fx_handle_t,
    ) -> sys::fx_status_t {
        let obj = object::ProcessObject(object::KernelObject);

        (self.cb)(&obj);

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
        0
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
fn fx_test_call() {
    context::with_logger(|c| info!("{:?}", c));
}

#[inline]
fn fx_create_process<SF, R>(f: SF, test: String)
where
    SF: FnOnce() -> R,
{
    // Make sure to save the guard, see documentation for more information
    let _guard = context::ScopeGuard::new(context::Context { test });
    f();
}

fn bar() {
    debug!("fn bar");
    fx_test_call();
}

fn foo() {
    debug!("fn foo");
    fx_test_call();

    // some bytes, in a vector
    let sparkle_heart = vec![240, 159, 146, 150];

    // We know these bytes are valid, so we'll use `unwrap()`.
    let sparkle_heart = String::from_utf8(sparkle_heart).unwrap();

    fx_create_process(|| bar(), String::from("bar"));
}

impl Kernel {
    pub fn new(on_process_start_cb: fn(&object::ProcessObject)) -> Self {
        Self {
            cb: on_process_start_cb,
        }
    }

    pub fn init(&self) {
        info!("initializing kernel");

        fx_create_process(|| foo(), String::from("foo"));
    }
}

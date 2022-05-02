// Copyright 2022 MeshX Contributors. All rights reserved.

mod context;
mod koid;

use std::str::FromStr;

use fiber_sys as sys;

pub struct Kernel {}

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

    fn sys_process_exit(&self, retcode: i64) -> sys::fx_status_t {
        0
    }

    fn sys_process_start(
        &self,
        handle: sys::fx_handle_t,
        entry: sys::fx_vaddr_t,
        arg1: sys::fx_handle_t,
    ) -> sys::fx_status_t {
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

    fn sys_process_create(
        &self,
        job: sys::fx_handle_t,
        name: *const u8,
        name_size: usize,
        options: u32,
        proc_handle: *mut sys::fx_handle_t,
        vmar_handle: *mut sys::fx_handle_t,
    ) -> sys::fx_status_t {
        0
    }
}

#[inline]
fn fx_test_call() {
    context::with_logger(|c| print!("{:?} \n", c));
}

#[inline]
fn fx_create_process<SF, R>(f: SF, test: String)
where
    SF: FnOnce() -> R,
{
    // Make sure to save the guard, see documentation for more information
    let _guard = context::ScopeGuard::new(&context::Context { test });
    f();
}

fn bar() {
    fx_test_call();
}

fn foo() {
    fx_test_call();

    // some bytes, in a vector
    let sparkle_heart = vec![240, 159, 146, 150];

    // We know these bytes are valid, so we'll use `unwrap()`.
    let sparkle_heart = String::from_utf8(sparkle_heart).unwrap();

    // Make sure to save the guard, see documentation for more information
    fx_create_process(|| bar(), sparkle_heart);
}

impl Kernel {
    pub fn new() -> Self {
        Self {}
    }

    pub fn init(&self) {
        println!("initializing mp");

        // Make sure to save the guard, see documentation for more information
        fx_create_process(|| bar(), String::from("tesadfdsfst"));
    }
}

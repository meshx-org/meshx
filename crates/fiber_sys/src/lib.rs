// Copyright 2022 MeshX Contributors. All rights reserved.
// Copyright 2019 The Fuchsia Authors. All rights reserved.
// Use of this source code is governed by a BSD-style license that can be
// found in the LICENSE file.

// re-export the types defined in the fiber_types crate
pub use fiber_types::*;

#[cfg(not(target_arch = "wasm32"))]
use once_cell::sync::OnceCell;

#[cfg(target_arch = "wasm32")]
extern "C" {
    // Handle calls
    pub fn fx_handle_close(handle: fx_handle_t) -> fx_status_t;
    pub fn fx_handle_duplicate(handle: fx_handle_t, rights: fx_rights_t, out: *const fx_handle_t) -> fx_status_t;
    pub fn fx_handle_replace(handle: fx_handle_t, rights: fx_rights_t, out: *const fx_handle_t) -> fx_status_t;
    // Object calls
    pub fn fx_object_get_info(handle: fx_handle_t, topic: u32, buffer: *const u8, buffer_size: usize) -> fx_status_t;
    // Process calls
    fn fx_process_create(
        job: fx_handle_t,
        name: *const u8,
        name_size: usize,
        options: u32,
        proc_handle: *mut fx_handle_t,
        dv_handle: *mut fx_handle_t,
    ) -> fx_status_t;
    pub fn fx_process_start(handle: fx_handle_t, entry: fx_vaddr_t, arg1: fx_handle_t) -> fx_status_t;
    pub fn fx_process_exit(retcode: i64) -> fx_status_t;
    // DataObject
    pub fn fx_do_create(size: u64, options: u32, out: *mut fx_handle_t) -> fx_status_t;
    pub fn fx_do_create_child(
        handle: fx_handle_t,
        options: u32,
        offset: u64,
        size: u64,
        out: *mut fx_handle_t,
    ) -> fx_status_t;
    pub fn fx_do_get_size(handle: fx_handle_t, size: *mut u64) -> fx_status_t;
    pub fn fx_do_set_size(handle: fx_handle_t, size: u64) -> fx_status_t;
    // DataView calls
    pub fn fx_dv_read(handle: fx_handle_t, buffer: *mut u8, offset: u64, buffer_size: usize) -> fx_status_t;
    pub fn fx_dv_write(handle: fx_handle_t, buffer: *const u8, offset: u64, buffer_size: usize) -> fx_status_t;

}

#[cfg(not(target_arch = "wasm32"))]
pub trait System {
    // Handle operations
    fn sys_handle_close(&self, handle: fx_handle_t) -> fx_status_t;
    fn sys_handle_duplicate(&self, handle: fx_handle_t, rights: fx_rights_t, out: *const fx_handle_t) -> fx_status_t;
    fn sys_handle_replace(&self, handle: fx_handle_t, rights: fx_rights_t, out: *const fx_handle_t) -> fx_status_t;
    // Object operations
    fn sys_object_get_info(
        &self,
        handle: fx_handle_t,
        topic: u32,
        buffer: *const u8,
        buffer_size: usize,
    ) -> fx_status_t;
    // Process operations
    fn sys_process_create(
        &self,
        job: fx_handle_t,
        name: *const u8,
        name_size: usize,
        options: u32,
        proc_handle: *mut fx_handle_t,
        dv_handle: *mut fx_handle_t,
    ) -> fx_status_t;
    fn sys_process_start(&self, handle: fx_handle_t, entry: fx_vaddr_t, arg1: fx_handle_t) -> fx_status_t;
    fn sys_process_exit(&self, retcode: i64) -> fx_status_t;
    // Job operations
    fn sys_job_create(&self, parent_job: fx_handle_t, options: u32, out: *const fx_handle_t) -> fx_status_t;
    fn sys_job_set_critical(&self, job: fx_handle_t, options: u32, process: fx_handle_t) -> fx_status_t;
    fn sys_job_set_policy(
        &self,
        handle: fx_handle_t,
        options: u32,
        topic: u32,
        policy: *const u8,
        policy_size: u32,
    ) -> fx_status_t;
    // Task operations
    fn sys_task_kill(&self, handle: fx_handle_t) -> fx_status_t;
}

#[cfg(not(target_arch = "wasm32"))]
pub static SYSTEM: OnceCell<Box<(dyn System + Send + Sync)>> = OnceCell::new();

#[cfg(not(target_arch = "wasm32"))]
pub fn fx_handle_close(handle: fx_handle_t) -> fx_status_t {
    let sys = SYSTEM.get().expect("SYSTEM is not initialized");
    sys.sys_handle_close(handle)
}

#[cfg(not(target_arch = "wasm32"))]
pub fn fx_handle_duplicate(handle: fx_handle_t, rights: fx_rights_t, out: *const fx_handle_t) -> fx_status_t {
    let sys = SYSTEM.get().expect("SYSTEM is not initialized");
    sys.sys_handle_duplicate(handle, rights, out)
}

#[cfg(not(target_arch = "wasm32"))]
pub fn fx_handle_replace(handle: fx_handle_t, rights: fx_rights_t, out: *const fx_handle_t) -> fx_status_t {
    let sys = SYSTEM.get().expect("SYSTEM is not initialized");
    sys.sys_handle_replace(handle, rights, out)
}

#[cfg(not(target_arch = "wasm32"))]
pub fn fx_object_get_info(handle: fx_handle_t, topic: u32, buffer: *const u8, buffer_size: usize) -> fx_status_t {
    let sys = SYSTEM.get().expect("SYSTEM is not initialized");
    sys.sys_object_get_info(handle, topic, buffer, buffer_size)
}

#[cfg(not(target_arch = "wasm32"))]
pub fn fx_process_create(
    job: fx_handle_t,
    name: *const u8,
    name_size: usize,
    options: u32,
    proc_handle: *mut fx_handle_t,
    dv_handle: *mut fx_handle_t,
) -> fx_status_t {
    let sys = SYSTEM.get().expect("SYSTEM is not initialized");
    println!("start");
    let s = sys.sys_process_create(job, name, name_size, options, proc_handle, dv_handle);
    println!("end");

    s
}

#[cfg(not(target_arch = "wasm32"))]
pub fn fx_process_start(handle: fx_handle_t, entry: fx_vaddr_t, arg1: fx_handle_t) -> fx_status_t {
    let sys = SYSTEM.get().expect("SYSTEM is not initialized");
    sys.sys_process_start(handle, entry, arg1)
}

#[cfg(not(target_arch = "wasm32"))]
pub fn fx_process_exit(retcode: i64) -> fx_status_t {
    let sys = SYSTEM.get().expect("SYSTEM is not initialized");
    sys.sys_process_exit(retcode)
}

#[cfg(not(target_arch = "wasm32"))]
pub fn fx_job_create(parent_job: fx_handle_t, options: u32, out: *const fx_handle_t) -> fx_status_t {
    let sys = SYSTEM.get().expect("SYSTEM is not initialized");
    sys.sys_job_create(parent_job, options, out)
}

#[cfg(not(target_arch = "wasm32"))]
pub fn fx_job_set_critical(job: fx_handle_t, options: u32, process: fx_handle_t) -> fx_status_t {
    let sys = SYSTEM.get().expect("SYSTEM is not initialized");
    sys.sys_job_set_critical(job, options, process)
}

#[cfg(not(target_arch = "wasm32"))]
pub fn fx_job_set_policy(
    handle: fx_handle_t,
    options: u32,
    topic: u32,
    policy: *const u8,
    policy_size: u32,
) -> fx_status_t {
    let sys = SYSTEM.get().expect("SYSTEM is not initialized");
    sys.sys_job_set_policy(handle, options, topic, policy, policy_size)
}

#[cfg(not(target_arch = "wasm32"))]
pub fn fx_task_kill(handle: fx_handle_t) -> fx_status_t {
    let sys = SYSTEM.get().expect("SYSTEM is not initialized");
    sys.sys_task_kill(handle)
}

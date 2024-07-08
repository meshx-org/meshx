// Copyright 2023 MeshX Contributors. All rights reserved.
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
    pub fn fx_object_signal_peer(handle: fx_handle_t, clear_mask: u32, set_mask: u32) -> fx_status_t;
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
    // Channel syscalls
    pub fn fx_channel_create(options: u32, out0: *mut fx_handle_t, out1: *mut fx_handle_t) -> fx_status_t;
}

#[cfg(not(target_arch = "wasm32"))]
pub trait System: std::fmt::Debug {
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
    fn sys_object_signal_peer(&self, handle: fx_handle_t, clear_mask: u32, set_mask: u32) -> fx_status_t;
    fn sys_object_signal(&self, handle: fx_handle_t, clear_mask: u32, set_mask: u32) -> fx_status_t;
    fn sys_object_wait_one(
        &self,
        handle: fx_handle_t,
        waitfor: fx_signals_t,
        deadline: fx_time_t,
        observed: *mut fx_signals_t,
    ) -> fx_status_t;
    fn sys_object_wait_async(
        &self,
        handle: fx_handle_t,
        port_handle: fx_handle_t,
        key: u64,
        signals: fx_signals_t,
        options: u32,
    ) -> fx_status_t;
    // Channel operations
    fn sys_channel_create(&self, options: u32, out0: *mut fx_handle_t, out1: *mut fx_handle_t) -> fx_status_t;
    fn sys_channel_read(
        &self,
        handle: fx_handle_t,
        options: u32,
        bytes: *mut u8,
        handles: *mut fx_handle_t,
        num_bytes: usize,
        num_handles: u32,
        actual_bytes: *mut usize,
        actual_handles: *mut u32,
    ) -> fx_status_t;
    fn sys_channel_read_etc(
        &self,
        handle: fx_handle_t,
        options: u32,
        bytes: *mut u8,
        handles: *mut fx_handle_info_t,
        num_bytes: u32,
        num_handles: u32,
        actual_bytes: *mut u32,
        actual_handles: *mut u32,
    ) -> fx_status_t;
    fn sys_channel_write(
        &self,
        handle: fx_handle_t,
        options: u32,
        bytes: *const u8,
        num_bytes: u32,
        handles: *const fx_handle_t,
        num_handles: u32,
    ) -> fx_status_t;
    fn sys_channel_write_etc(
        &self,
        handle: fx_handle_t,
        options: u32,
        bytes: *const u8,
        num_bytes: u32,
        handles: *const fx_handle_disposition_t,
        num_handles: u32,
    ) -> fx_status_t;
    fn sys_channel_call_etc(
        &self,
        handle: fx_handle_t,
        options: u32,
        deadline: fx_time_t,
        args: *const fx_channel_call_etc_args_t,
        actual_bytes: *const u32,
        actual_handles: *const u32,
    ) -> fx_status_t;
    // VMO operations
    fn sys_vmo_create(&self, size: u64, options: u32, out: *mut fx_handle_t) -> fx_status_t;
    fn sys_vmo_read(&self, handle: fx_handle_t, buffer: *mut u8, offset: u64, buffer_size: usize) -> fx_status_t;
    fn sys_vmo_write(&self, handle: fx_handle_t, buffer: *const u8, offset: u64, buffer_size: usize) -> fx_status_t;
    fn sys_vmo_get_size(&self, handle: fx_handle_t, size: *mut u64) -> fx_status_t;
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
    // Port operations
    fn sys_port_create(&self, options: u32, out: *mut fx_handle_t) -> fx_status_t;
    fn sys_port_queue(&self, handle: fx_handle_t, packet: *const fx_port_packet_t) -> fx_status_t;
    fn sys_port_wait(&self, handle: fx_handle_t, deadline: fx_time_t, packet: *mut fx_port_packet_t) -> fx_status_t;
    fn sys_port_cancel(&self, handle: fx_handle_t, source: fx_handle_t, key: u64) -> fx_status_t;
}

#[cfg(all(not(test), not(target_arch = "wasm32")))]
pub static SYSTEM: OnceCell<std::sync::Arc<(dyn System + Send + Sync)>> = OnceCell::new();

#[cfg(all(not(test), not(target_arch = "wasm32")))]
pub fn fx_handle_close(handle: fx_handle_t) -> fx_status_t {
    let sys = SYSTEM.get().expect("SYSTEM is not initialized");
    sys.sys_handle_close(handle)
}

#[cfg(all(not(test), not(target_arch = "wasm32")))]
pub fn fx_handle_duplicate(handle: fx_handle_t, rights: fx_rights_t, out: *const fx_handle_t) -> fx_status_t {
    let sys = SYSTEM.get().expect("SYSTEM is not initialized");
    sys.sys_handle_duplicate(handle, rights, out)
}

#[cfg(all(not(test), not(target_arch = "wasm32")))]
pub fn fx_handle_replace(handle: fx_handle_t, rights: fx_rights_t, out: *const fx_handle_t) -> fx_status_t {
    let sys = SYSTEM.get().expect("SYSTEM is not initialized");
    sys.sys_handle_replace(handle, rights, out)
}

#[cfg(all(not(test), not(target_arch = "wasm32")))]
pub fn fx_object_get_info(handle: fx_handle_t, topic: u32, buffer: *const u8, buffer_size: usize) -> fx_status_t {
    let sys = SYSTEM.get().expect("SYSTEM is not initialized");
    sys.sys_object_get_info(handle, topic, buffer, buffer_size)
}

#[cfg(all(not(test), not(target_arch = "wasm32")))]
pub fn fx_object_signal_peer(handle: fx_handle_t, clear_mask: u32, set_mask: u32) -> fx_status_t {
    let sys = SYSTEM.get().expect("SYSTEM is not initialized");
    sys.sys_object_signal_peer(handle, clear_mask, set_mask)
}

#[cfg(all(not(test), not(target_arch = "wasm32")))]
pub fn fx_object_wait_one(
    handle: fx_handle_t,
    waitfor: fx_signals_t,
    deadline: fx_time_t,
    observed: *mut fx_signals_t,
) -> fx_status_t {
    let sys = SYSTEM.get().expect("SYSTEM is not initialized");
    sys.sys_object_wait_one(handle, waitfor, deadline, observed)
}

#[cfg(all(not(test), not(target_arch = "wasm32")))]
pub fn fx_object_signal(handle: fx_handle_t, clear_mask: u32, set_mask: u32) -> fx_status_t {
    let sys = SYSTEM.get().expect("SYSTEM is not initialized");
    sys.sys_object_signal(handle, clear_mask, set_mask)
}

#[cfg(all(not(test), not(target_arch = "wasm32")))]
pub fn fx_object_wait_async(
    handle: fx_handle_t,
    port_handle: fx_handle_t,
    key: u64,
    signals: fx_signals_t,
    options: u32,
) -> fx_status_t {
    let sys = SYSTEM.get().expect("SYSTEM is not initialized");
    sys.sys_object_wait_async(handle, port_handle, key, signals, options)
}

#[cfg(all(not(test), not(target_arch = "wasm32")))]
pub fn fx_channel_create(options: u32, out0: *mut fx_handle_t, out1: *mut fx_handle_t) -> fx_status_t {
    let sys = SYSTEM.get().expect("SYSTEM is not initialized");
    sys.sys_channel_create(options, out0, out1)
}

#[cfg(all(not(test), not(target_arch = "wasm32")))]
pub fn fx_process_create(
    job: fx_handle_t,
    name: *const u8,
    name_size: usize,
    options: u32,
    proc_handle: *mut fx_handle_t,
    dv_handle: *mut fx_handle_t,
) -> fx_status_t {
    let sys = SYSTEM.get().expect("SYSTEM is not initialized");
    sys.sys_process_create(job, name, name_size, options, proc_handle, dv_handle)
}

#[cfg(all(not(test), not(target_arch = "wasm32")))]
pub fn fx_process_start(handle: fx_handle_t, entry: fx_vaddr_t, arg1: fx_handle_t) -> fx_status_t {
    let sys = SYSTEM.get().expect("SYSTEM is not initialized");
    sys.sys_process_start(handle, entry, arg1)
}

#[cfg(all(not(test), not(target_arch = "wasm32")))]
pub fn fx_process_exit(retcode: i64) -> fx_status_t {
    let sys = SYSTEM.get().expect("SYSTEM is not initialized");
    sys.sys_process_exit(retcode)
}

#[cfg(all(not(test), not(target_arch = "wasm32")))]
pub fn fx_job_create(parent_job: fx_handle_t, options: u32, out: *const fx_handle_t) -> fx_status_t {
    let sys = SYSTEM.get().expect("SYSTEM is not initialized");
    sys.sys_job_create(parent_job, options, out)
}

#[cfg(all(not(test), not(target_arch = "wasm32")))]
pub fn fx_job_set_critical(job: fx_handle_t, options: u32, process: fx_handle_t) -> fx_status_t {
    let sys = SYSTEM.get().expect("SYSTEM is not initialized");
    sys.sys_job_set_critical(job, options, process)
}

#[cfg(all(not(test), not(target_arch = "wasm32")))]
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

#[cfg(all(not(test), not(target_arch = "wasm32")))]
pub fn fx_task_kill(handle: fx_handle_t) -> fx_status_t {
    let sys = SYSTEM.get().expect("SYSTEM is not initialized");
    sys.sys_task_kill(handle)
}

#[cfg(all(not(test), not(target_arch = "wasm32")))]
pub fn fx_port_create(options: u32, out: *mut fx_handle_t) -> fx_status_t {
    let sys = SYSTEM.get().expect("SYSTEM is not initialized");
    sys.sys_port_create(options, out)
}

#[cfg(all(not(test), not(target_arch = "wasm32")))]
pub fn fx_port_queue(handle: fx_handle_t, packet: *const fx_port_packet_t) -> fx_status_t {
    let sys = SYSTEM.get().expect("SYSTEM is not initialized");
    sys.sys_port_queue(handle, packet)
}

#[cfg(all(not(test), not(target_arch = "wasm32")))]
pub fn fx_port_wait(handle: fx_handle_t, deadline: fx_time_t, packet: *mut fx_port_packet_t) -> fx_status_t {
    let sys = SYSTEM.get().expect("SYSTEM is not initialized");
    sys.sys_port_wait(handle, deadline, packet)
}

#[cfg(all(not(test), not(target_arch = "wasm32")))]
pub fn fx_port_cancel(handle: fx_handle_t, source: fx_handle_t, key: u64) -> fx_status_t {
    let sys = SYSTEM.get().expect("SYSTEM is not initialized");
    sys.sys_port_cancel(handle, source, key)
}

#[cfg(all(not(test), not(target_arch = "wasm32")))]
pub fn fx_channel_read(
    handle: fx_handle_t,
    options: u32,
    bytes: *mut u8,
    handles: *mut fx_handle_t,
    num_bytes: usize,
    num_handles: u32,
    actual_bytes: *mut usize,
    actual_handles: *mut u32,
) -> fx_status_t {
    let sys = SYSTEM.get().expect("SYSTEM is not initialized");
    sys.sys_channel_read(
        handle,
        options,
        bytes,
        handles,
        num_bytes,
        num_handles,
        actual_bytes,
        actual_handles,
    )
}

#[cfg(all(not(test), not(target_arch = "wasm32")))]
pub fn fx_channel_read_etc(
    handle: fx_handle_t,
    options: u32,
    bytes: *mut u8,
    handles: *mut fx_handle_info_t,
    num_bytes: u32,
    num_handles: u32,
    actual_bytes: *mut u32,
    actual_handles: *mut u32,
) -> fx_status_t {
    let sys = SYSTEM.get().expect("SYSTEM is not initialized");
    sys.sys_channel_read_etc(
        handle,
        options,
        bytes,
        handles,
        num_bytes,
        num_handles,
        actual_bytes,
        actual_handles,
    )
}

#[cfg(all(not(test), not(target_arch = "wasm32")))]
pub fn fx_channel_write(
    handle: fx_handle_t,
    options: u32,
    bytes: *const u8,
    num_bytes: u32,
    handles: *const fx_handle_t,
    num_handles: u32,
) -> fx_status_t {
    let sys = SYSTEM.get().expect("SYSTEM is not initialized");
    sys.sys_channel_write(handle, options, bytes, num_bytes, handles, num_handles)
}

#[cfg(all(not(test), not(target_arch = "wasm32")))]
pub fn fx_channel_write_etc(
    handle: fx_handle_t,
    options: u32,
    bytes: *const u8,
    num_bytes: u32,
    handles: *const fx_handle_disposition_t,
    num_handles: u32,
) -> fx_status_t {
    let sys = SYSTEM.get().expect("SYSTEM is not initialized");
    sys.sys_channel_write_etc(handle, options, bytes, num_bytes, handles, num_handles)
}

#[cfg(all(not(test), not(target_arch = "wasm32")))]
pub fn fx_channel_call_etc(
    handle: fx_handle_t,
    options: u32,
    deadline: fx_time_t,
    args: *const fx_channel_call_etc_args_t,
    actual_bytes: *const u32,
    actual_handles: *const u32,
) -> fx_status_t {
    let sys = SYSTEM.get().expect("SYSTEM is not initialized");
    sys.sys_channel_call_etc(handle, options, deadline, args, actual_bytes, actual_handles)
}

#[cfg(all(test, not(target_arch = "wasm32")))]
pub fn fx_vmo_create(size: u64, options: u32, out: *mut fx_handle_t) -> fx_status_t {
   print!("fx_vmo_create");
   0
}

#[cfg(all(not(test), not(target_arch = "wasm32")))]
pub fn fx_vmo_create(size: u64, options: u32, out: *mut fx_handle_t) -> fx_status_t {
    let sys = SYSTEM.get().expect("SYSTEM is not initialized");
    sys.sys_vmo_create(size, options, out)
}

#[cfg(all(not(test), not(target_arch = "wasm32")))]
pub fn fx_vmo_read(handle: fx_handle_t, buffer: *mut u8, offset: u64, buffer_size: usize) -> fx_status_t {
    let sys = SYSTEM.get().expect("SYSTEM is not initialized");
    sys.sys_vmo_read(handle, buffer, offset, buffer_size)
}

#[cfg(all(not(test), not(target_arch = "wasm32")))]
pub fn fx_vmo_write(handle: fx_handle_t, buffer: *const u8, offset: u64, buffer_size: usize) -> fx_status_t {
    let sys = SYSTEM.get().expect("SYSTEM is not initialized");
    sys.sys_vmo_write(handle, buffer, offset, buffer_size)
}

#[cfg(all(not(test), not(target_arch = "wasm32")))]
pub fn fx_vmo_get_size(handle: fx_handle_t, size: *mut u64) -> fx_status_t {
    let sys = SYSTEM.get().expect("SYSTEM is not initialized");
    sys.sys_vmo_get_size(handle, size)
}

#[cfg(all(not(test), not(target_arch = "wasm32")))]
pub fn fx_clock_get_monotonic() -> fx_time_t {
    0
}

#[cfg(all(not(test), not(target_arch = "wasm32")))]
pub fn fx_ticks_get() -> fx_ticks_t {
    0
}

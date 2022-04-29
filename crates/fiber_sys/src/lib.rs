// Copyright 2022 MeshX Contributors. All rights reserved.
// Copyright 2019 The Fuchsia Authors. All rights reserved.
// Use of this source code is governed by a BSD-style license that can be
// found in the LICENSE file.
// WARNING: THIS FILE IS MACHINE GENERATED BY //tools/kazoo. DO NOT EDIT.

// re-export the types defined in the fiber_types crate
pub use fiber_types::*;

#[cfg(not(target_arch = "wasm32"))]
use once_cell::sync::OnceCell;

#[cfg(target_arch = "wasm32")]
extern "C" {
    pub fn fx_handle_close(handle: fx_handle_t) -> fx_status_t;
    pub fn fx_handle_duplicate(handle: fx_handle_t, rights: fx_rights_t, out: *const fx_handle_t) -> fx_status_t;
    pub fn fx_handle_replace(handle: fx_handle_t, rights: fx_rights_t, out: *const fx_handle_t) -> fx_status_t;
    pub fn fx_object_get_info(
        &self,
        handle: fx_handle_t,
        topic: u32,
        buffer: *const u8,
        buffer_size: usize,
    ) -> fx_status_t;
    pub fn fx_process_start(handle: fx_handle_t, entry: fx_vaddr_t, arg1: fx_handle_t) -> fx_status_t;
    pub fn fx_process_exit(retcode: i64) -> fx_status_t;
}

#[cfg(not(target_arch = "wasm32"))]
pub trait System {
    fn sys_handle_close(&self, handle: fx_handle_t) -> fx_status_t;
    fn sys_handle_duplicate(&self, handle: fx_handle_t, rights: fx_rights_t, out: *const fx_handle_t) -> fx_status_t;
    fn sys_handle_replace(&self, handle: fx_handle_t, rights: fx_rights_t, out: *const fx_handle_t) -> fx_status_t;
    fn sys_object_get_info(
        &self,
        handle: fx_handle_t,
        topic: u32,
        buffer: *const u8,
        buffer_size: usize,
    ) -> fx_status_t;
    fn sys_process_exit(&self, retcode: i64) -> fx_status_t;
    fn sys_process_start(&self, handle: fx_handle_t, entry: fx_vaddr_t, arg1: fx_handle_t) -> fx_status_t;
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
pub fn fx_process_start(handle: fx_handle_t, entry: fx_vaddr_t, arg1: fx_handle_t) -> fx_status_t {
    let sys = SYSTEM.get().expect("SYSTEM is not initialized");
    sys.sys_process_start(handle, entry, arg1)
}

#[cfg(not(target_arch = "wasm32"))]
pub fn fx_process_exit(retcode: i64) -> fx_status_t {
    let sys = SYSTEM.get().expect("SYSTEM is not initialized");
    sys.sys_process_exit(retcode)
}

// Copyright 2022 MeshX Contributors. All rights reserved.

use fiber_sys as sys;

pub struct Kernel;

impl fiber_sys::System for Kernel {
    fn sys_handle_close(&self, handle: sys::fx_handle_t) -> sys::fx_status_t {
        todo!()
    }

    fn sys_handle_duplicate(
        &self,
        handle: sys::fx_handle_t,
        rights: sys::fx_rights_t,
        out: *const sys::fx_handle_t,
    ) -> sys::fx_status_t {
        todo!()
    }

    fn sys_handle_replace(
        &self,
        handle: sys::fx_handle_t,
        rights: sys::fx_rights_t,
        out: *const sys::fx_handle_t,
    ) -> sys::fx_status_t {
        todo!()
    }

    fn sys_object_get_info(
        &self,
        handle: sys::fx_handle_t,
        topic: u32,
        buffer: *const u8,
        buffer_size: usize,
    ) -> sys::fx_status_t {
        todo!()
    }

    fn sys_process_exit(&self, retcode: i64) -> sys::fx_status_t {
        todo!()
    }

    fn sys_process_start(
        &self,
        handle: sys::fx_handle_t,
        entry: sys::fx_vaddr_t,
        arg1: sys::fx_handle_t,
    ) -> sys::fx_status_t {
        todo!()
    }
}

impl Kernel {
    pub fn new() -> Self {
        Self {}
    }
}

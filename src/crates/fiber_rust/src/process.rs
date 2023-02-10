// Copyright 2022 MeshX Contributors. All rights reserved.
// Copyright 2017 The Fuchsia Authors. All rights reserved.
// Use of this source code is governed by a BSD-style license that can be
// found in the LICENSE file.

//! Type-safe bindings for Zircon processes.

use fiber_sys as sys;

use crate::handle::{Handle, HandleBased, HandleRef, AsHandleRef};
use crate::impl_handle_based;
use crate::status::Status;

/// An object representing a Zircon process.
///
/// As essentially a subtype of `Handle`, it can be freely interconverted.
#[derive(Debug, Eq, PartialEq, Ord, PartialOrd, Hash)]
#[repr(transparent)]
pub struct Process(Handle);

impl_handle_based!(Process);

impl Process {
    /// Similar to `Thread::start`, but is used to start the first thread in a process.
    ///
    /// Wraps the
    /// [zx_process_start](https://fuchsia.dev/fuchsia-src/reference/syscalls/process_start.md)
    /// syscall.
    pub fn start(&self, entry: usize, arg1: Handle) -> Result<(), Status> {
        let process_raw = self.raw_handle();
        let arg1 = arg1.into_raw();
        Status::ok(unsafe { sys::fx_process_start(process_raw, entry, arg1) })
    }

    // TODO: process object infos

    /// Exit the current process with the given return code.
    ///
    /// Wraps the
    /// [zx_process_exit](https://fuchsia.dev/fuchsia-src/reference/syscalls/process_exit.md)
    /// syscall.
    pub fn exit(retcode: i64) -> ! {
        unsafe {
            sys::fx_process_exit(retcode);
            // kazoo generates the syscall returning a unit value. We know it will not proceed
            // past this point however.
            std::hint::unreachable_unchecked()
        }
    }
}

// impl Task for Process {}

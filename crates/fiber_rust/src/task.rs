// Copyright 2019 The Fuchsia Authors. All rights reserved.
// Use of this source code is governed by a BSD-style license that can be
// found in the LICENSE file.
use crate::{object_get_info, ok};
use crate::{AsHandleRef, Handle, ObjectQuery, Status, Topic};
use bitflags::bitflags;
use fiber_sys as sys;

pub trait Task: AsHandleRef {
    /// Kill the given task (job, process, or thread).
    ///
    /// Wraps the
    /// [zx_task_kill](https://fuchsia.dev/fuchsia-src/reference/syscalls/task_kill.md)
    /// syscall.
    // TODO(fxbug.dev/72722): guaranteed to return an error when called on a Thread.
    fn kill(&self) -> Result<(), Status> {
        ok(unsafe { sys::fx_task_kill(self.raw_handle()) })
    }
}

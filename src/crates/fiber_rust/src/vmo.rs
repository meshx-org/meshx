// Copyright 2023 MeshX Contributors. All rights reserved.
// Copyright 2017 The Fuchsia Authors. All rights reserved.
// Use of this source code is governed by a BSD-style license that can be
// found in the LICENSE file.

//! Type-safe bindings for Zircon vmo objects.

use crate::{impl_handle_based, ok};
use crate::{object_get_info, ObjectQuery, Topic};
use crate::{AsHandleRef, Handle, HandleBased, HandleRef, Koid, Rights, Status};
use bitflags::bitflags;
use fiber_sys as sys;
use fiber_types as fx;

/// An object representing a Zircon
/// [virtual memory object](https://fuchsia.dev/fuchsia-src/concepts/objects/vm_object.md).
///
/// As essentially a subtype of `Handle`, it can be freely interconverted.
#[derive(Debug, Eq, PartialEq, Ord, PartialOrd, Hash)]
#[repr(transparent)]
pub struct Vmo(Handle);
impl_handle_based!(Vmo);

impl Vmo {
    /// Create a virtual memory object.
    ///
    /// Wraps the
    /// `zx_vmo_create` syscall. See the [Shared Memory: Virtual Memory Objects (VMOs)](https://fuchsia.dev/fuchsia-src/concepts/kernel/concepts#shared_memory_virtual_memory_objects_vmos)
    /// for more information.
    pub fn create(size: u64) -> Result<Vmo, Status> {
        Vmo::create_with_opts(VmoOptions::from_bits_truncate(0), size)
    }

    /// Create a virtual memory object with options.
    ///
    /// Wraps the `fx_vmo_create` syscall, allowing options to be passed.
    pub fn create_with_opts(opts: VmoOptions, size: u64) -> Result<Vmo, Status> {
        let mut handle = 0;
        let status = unsafe { sys::fx_vmo_create(size, opts.bits(), &mut handle) };
        ok(status)?;
        unsafe { Ok(Vmo::from(Handle::from_raw(handle))) }
    }

    /// Get the size of a virtual memory object.
    ///
    /// Wraps the `fx_vmo_get_size` syscall.
    pub fn get_size(&self) -> Result<u64, Status> {
        let mut size = 0;
        let status = unsafe { sys::fx_vmo_get_size(self.raw_handle(), &mut size) };
        ok(status).map(|()| size)
    }

    /// Read from a virtual memory object.
    ///
    /// Wraps the `fx_vmo_read` syscall.
    pub fn read(&self, data: &mut [u8], offset: u64) -> Result<(), Status> {
        unsafe {
            let status = sys::fx_vmo_read(self.raw_handle(), data.as_mut_ptr(), offset, data.len());
            ok(status)
        }
    }

    /// Write to a virtual memory object.
    ///
    /// Wraps the `fx_vmo_write` syscall.
    pub fn write(&self, data: &[u8], offset: u64) -> Result<(), Status> {
        unsafe {
            let status = sys::fx_vmo_write(self.raw_handle(), data.as_ptr(), offset, data.len());
            ok(status)
        }
    }
}

bitflags! {
    /// Options that may be used when creating a `Vmo`.
    #[repr(transparent)]
    pub struct VmoOptions: u32 {
        const RESIZABLE = fx::FX_VMO_RESIZABLE;
        const TRAP_DIRTY = fx::FX_VMO_TRAP_DIRTY;
    }
}

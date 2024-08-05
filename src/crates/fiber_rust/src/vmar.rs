// Copyright 2023 MeshX Contributors. All rights reserved.
// Copyright 2017 The Fuchsia Authors. All rights reserved.
// Use of this source code is governed by a BSD-style license that can be
// found in the LICENSE file.

//! Type-safe bindings for Zircon vmar objects.

use crate::{impl_handle_based, ok};
//use crate::{object_get_info, ObjectQuery, Topic};
use crate::{AsHandleRef, Handle, HandleBased, HandleRef, Status};
use bitflags::bitflags;
use fiber_sys as sys;

/// An object representing a Zircon
/// [virtual memory address region](https://fuchsia.dev/fuchsia-src/concepts/objects/vm_address_region.md).
///
/// As essentially a subtype of `Handle`, it can be freely interconverted.

#[derive(Debug, Eq, PartialEq, Ord, PartialOrd, Hash)]
#[repr(transparent)]
pub struct Vmar(Handle);

impl_handle_based!(Vmar);

impl Vmar {
   
}

// TODO(smklein): Ideally we would have two separate sets of bitflags,
// and a union of both of them.
macro_rules! vmar_flags {
    (
        safe: [$($safe_name:ident : $safe_sys_name:ident,)*],
        extended: [$($ex_name:ident : $ex_sys_name:ident,)*],
    ) => {
        bitflags! {
            /// Flags to VMAR routines which are considered safe.
            #[repr(transparent)]
            pub struct VmarFlags: sys::fx_vm_option_t {
                $(
                    const $safe_name = sys::$safe_sys_name;
                )*
            }
        }
        bitflags! {
            /// Flags to all VMAR routines.
            #[repr(transparent)]
            pub struct VmarFlagsExtended: sys::fx_vm_option_t {
                $(
                    const $safe_name = sys::$safe_sys_name;
                )*
                $(
                    const $ex_name = sys::$ex_sys_name;
                )*
            }
        }
    };
}

vmar_flags! {
    safe: [
        PERM_READ: FX_VM_PERM_READ,
        PERM_WRITE: FX_VM_PERM_WRITE,
        PERM_EXECUTE: FX_VM_PERM_EXECUTE,
        COMPACT: FX_VM_COMPACT,
        SPECIFIC: FX_VM_SPECIFIC,
        CAN_MAP_SPECIFIC: FX_VM_CAN_MAP_SPECIFIC,
        CAN_MAP_READ: FX_VM_CAN_MAP_READ,
        CAN_MAP_WRITE: FX_VM_CAN_MAP_WRITE,
        CAN_MAP_EXECUTE: FX_VM_CAN_MAP_EXECUTE,
        MAP_RANGE: FX_VM_MAP_RANGE,
        REQUIRE_NON_RESIZABLE: FX_VM_REQUIRE_NON_RESIZABLE,
        ALLOW_FAULTS: FX_VM_ALLOW_FAULTS,
        OFFSET_IS_UPPER_LIMIT: FX_VM_OFFSET_IS_UPPER_LIMIT,
    ],
    extended: [
        SPECIFIC_OVERWRITE: FX_VM_SPECIFIC_OVERWRITE,
    ],
}

/*#[cfg(test)]
mod tests {
    // The unit tests are built with a different crate name, but fuchsia_runtime returns a "real"
    // fuchsia_zircon::Vmar that we need to use.
    use fuchsia_zircon::{Status, VmarFlags};

    #[test]
    fn allocate_and_info() -> Result<(), Status> {
        let size = usize::pow(2, 20); // 1MiB
        let root_vmar = fuchsia_runtime::vmar_root_self();
        let (vmar, base) = root_vmar.allocate(0, size, VmarFlags::empty())?;
        let info = vmar.info()?;
        assert!(info.base == base);
        assert!(info.len == size);
        Ok(())
    }

    #[test]
    fn root_vmar_info() -> Result<(), Status> {
        let root_vmar = fuchsia_runtime::vmar_root_self();
        let info = root_vmar.info()?;
        assert!(info.base > 0);
        assert!(info.len > 0);
        Ok(())
    }
}*/

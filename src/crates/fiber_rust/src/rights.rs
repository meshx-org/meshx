// Copyright 2023 MeshX Contributors. All rights reserved.
// Copyright 2018 The Fuchsia Authors. All rights reserved.
// Use of this source code is governed by a BSD-style license that can be
// found in the LICENSE file.

//! Type-safe bindings for Zircon rights.
use bitflags::bitflags;
use fiber_sys as sys;

bitflags! {
    /// Rights associated with a handle.
    ///
    /// See [rights](https://fuchsia.dev/fuchsia-src/concepts/kernel/rights) for more information.
    #[repr(C)]
    pub struct Rights: sys::fx_rights_t {
        const NONE            = sys::FX_RIGHT_NONE;
        const DUPLICATE       = sys::FX_RIGHT_DUPLICATE;
        const TRANSFER        = sys::FX_RIGHT_TRANSFER;
        const READ            = sys::FX_RIGHT_READ;
        const WRITE           = sys::FX_RIGHT_WRITE;
        const EXECUTE         = sys::FX_RIGHT_EXECUTE;
        const MAP             = sys::FX_RIGHT_MAP;
        const GET_PROPERTY    = sys::FX_RIGHT_GET_PROPERTY;
        const SET_PROPERTY    = sys::FX_RIGHT_SET_PROPERTY;
        const ENUMERATE       = sys::FX_RIGHT_ENUMERATE;
        const DESTROY         = sys::FX_RIGHT_DESTROY;
        const SET_POLICY      = sys::FX_RIGHT_SET_POLICY;
        const GET_POLICY      = sys::FX_RIGHT_GET_POLICY;
        const SIGNAL          = sys::FX_RIGHT_SIGNAL;
        const SIGNAL_PEER     = sys::FX_RIGHT_SIGNAL_PEER;
        const WAIT            = sys::FX_RIGHT_WAIT;
        const INSPECT         = sys::FX_RIGHT_INSPECT;
        const MANAGE_JOB      = sys::FX_RIGHT_MANAGE_JOB;
        const MANAGE_PROCESS  = sys::FX_RIGHT_MANAGE_PROCESS;
        const MANAGE_THREAD   = sys::FX_RIGHT_MANAGE_THREAD;
        const APPLY_PROFILE   = sys::FX_RIGHT_APPLY_PROFILE;
        const MANAGE_SOCKET   = sys::FX_RIGHT_MANAGE_SOCKET;
        // const RESIZE          = sys::FX_RIGHT_RESIZE;
        const SAME_RIGHTS     = sys::FX_RIGHT_SAME_RIGHTS;
        const BASIC           = sys::FX_RIGHT_TRANSFER | sys::FX_RIGHT_DUPLICATE |
                                sys::FX_RIGHT_WAIT | sys::FX_RIGHT_INSPECT;
        const IO              = sys::FX_RIGHT_READ | sys::FX_RIGHT_WRITE;
        const PROPERTY        = sys::FX_RIGHT_GET_PROPERTY | sys::FX_RIGHT_SET_PROPERTY;
        const POLICY          = sys::FX_RIGHT_GET_POLICY | sys::FX_RIGHT_SET_POLICY;
        const RESOURCE_BASIC  = sys::FX_RIGHT_TRANSFER | sys::FX_RIGHT_DUPLICATE |
                                sys::FX_RIGHT_WRITE | sys::FX_RIGHT_INSPECT;
        const CHANNEL_DEFAULT = sys::FX_RIGHT_TRANSFER | sys::FX_RIGHT_WAIT |
                                sys::FX_RIGHT_INSPECT |sys::FX_RIGHT_READ |
                                sys::FX_RIGHT_WRITE | sys::FX_RIGHT_SIGNAL |
                                sys::FX_RIGHT_SIGNAL_PEER;
        const VMO_DEFAULT     = Self::BASIC.bits() | Self::IO.bits() | Self::PROPERTY.bits() | Self::MAP.bits() | sys::FX_RIGHT_SIGNAL;
    }
}

impl Rights {
    /// Same as from_bits() but a const fn.
    #[inline]
    pub const fn from_bits_const(bits: sys::fx_rights_t) -> Option<Rights> {
        if (bits & !Rights::all().bits()) == 0 {
            return Some(Rights { bits });
        } else {
            None
        }
    }
}
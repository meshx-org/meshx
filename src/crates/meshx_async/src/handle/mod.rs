// Copyright 2023 MeshX Contributors. All rights reserved.
// Copyright 2020 The Fuchsia Authors. All rights reserved.
// Use of this source code is governed by a BSD-style license that can be
// found in the LICENSE file.

pub mod channel;
pub mod on_signals;
pub mod rwhandle;

/// invoke_for_handle_types!{mmm} calls the macro `mmm!` with two arguments: one is the name of a
/// Zircon handle, the second is one of:
///   * Everywhere for handle types that are supported everywhere FIDL is
///   * FuchsiaOnly for handle types that are supported only on Fuchsia
///   * Stub for handle types that have not yet had a Fuchsia API implemented in the zircon crate
///
/// To make a handle available everywhere, a polyfill must be implemented in
/// crate::handle::emulated.
#[macro_export]
macro_rules! invoke_for_handle_types {
    ($x:ident) => {
        $x! {Process, "Process", PROCESS, ZX_OBJ_TYPE_PROCESS, FuchsiaOnly}
        $x! {Channel, "Channel", CHANNEL, ZX_OBJ_TYPE_CHANNEL, Everywhere}
        // $x! {Event, "Event", EVENT, ZX_OBJ_TYPE_EVENT, Everywhere}
        $x! {Port, "Port", PORT, ZX_OBJ_TYPE_PORT, FuchsiaOnly}
        $x! {Job, "Job", JOB, ZX_OBJ_TYPE_JOB, FuchsiaOnly}
        $x! {Vmo, "Vmo", VMO, ZX_OBJ_TYPE_VMO, FuchsiaOnly}
        // $x! {Vmar, "VMAR", VMAR, ZX_OBJ_TYPE_VMAR, FuchsiaOnly}
    };
}
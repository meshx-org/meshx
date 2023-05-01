// Copyright 2023 MeshX Contributors. All rights reserved.
// Copyright 2016 The Fuchsia Authors. All rights reserved.
// Use of this source code is governed by a BSD-style license that can be
// found in the LICENSE file.

//! Library and runtime for fidl bindings.

#![deny(missing_docs)]
#![allow(elided_lifetimes_in_paths)]

#[macro_use]
pub mod encoding;

pub mod client;
pub mod endpoints;
pub mod epitaph;
pub mod handle;
pub mod prelude;
pub mod server;

mod error;
pub use self::error::{Error, Result};

pub use handle::*;
pub use server::ServeInner;

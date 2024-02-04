// Copyright 2020 The Fuchsia Authors. All rights reserved.
// Use of this source code is governed by a BSD-style license that can be
// found in the LICENSE file.

//! Utility functions for meshx.io files.

//mod async_reader;
//pub use async_reader::AsyncReader;

mod async_read_at;
pub use async_read_at::{Adapter, AsyncFile, AsyncGetSize, AsyncGetSizeExt, AsyncReadAt};
mod async_read_at_ext;
pub use async_read_at_ext::AsyncReadAtExt;
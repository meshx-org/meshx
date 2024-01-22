// Copyright 2019 The Fuchsia Authors. All rights reserved.
// Use of this source code is governed by a BSD-style license that can be
// found in the LICENSE file.

//! `EntryContainer` is a trait implemented by directories that allow manipulation of their
//! content.

use async_trait::async_trait;

use crate::{directory::entry::DirectoryEntry, node::Node};

/// All directories implement this trait.  If a directory can be modified it should
/// also implement the `MutableDirectory` trait.
#[async_trait]
pub trait Directory: DirectoryEntry + Node {}

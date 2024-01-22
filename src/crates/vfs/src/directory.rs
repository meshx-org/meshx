// Copyright 2019 The Fuchsia Authors. All rights reserved.
// Use of this source code is governed by a BSD-style license that can be
// found in the LICENSE file.

//! Module holding different kinds of pseudo directories and their building blocks.

pub mod mutable;
pub mod immutable;

pub mod entry;
pub mod entry_container;
pub mod simple;
pub mod connection;

/// A directory can be open either as a directory or a node.
#[derive(Clone)]
pub struct DirectoryOptions {
    // pub(crate) rights: fio::Operations,
}

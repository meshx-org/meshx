// Copyright 2023 The Fuchsia Authors. All rights reserved.
// Use of this source code is governed by a BSD-style license that can be
// found in the LICENSE file.

//! Implementation of a (limited) node connection.

use std::sync::Arc;

use crate::directory::entry::DirectoryEntry;
use async_trait::async_trait;

/// All nodes must implement this trait.
#[async_trait]
pub trait Node: DirectoryEntry {
    /// Called when the node is closed.
    fn close(self: Arc<Self>) {}
}

/// This struct is a RAII wrapper around a node that will call close() on it when dropped.
pub struct OpenNode<T: Node + ?Sized> {
    node: Arc<T>,
}

impl<T: Node + ?Sized> OpenNode<T> {
    pub fn new(node: Arc<T>) -> Self {
        Self { node }
    }
}

impl<T: Node + ?Sized> Drop for OpenNode<T> {
    fn drop(&mut self) {
        self.node.clone().close();
    }
}

impl<T: Node + ?Sized> std::ops::Deref for OpenNode<T> {
    type Target = Arc<T>;

    fn deref(&self) -> &Self::Target {
        &self.node
    }
}

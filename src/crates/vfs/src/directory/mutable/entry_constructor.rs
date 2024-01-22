// Copyright 2019 The Fuchsia Authors. All rights reserved.
// Use of this source code is governed by a BSD-style license that can be
// found in the LICENSE file.

//! Mutable directories need to create new entries inside themselves.  [`EntryConstructor`] is a
//! mechanism that allows the library user to control this process.

use crate::{directory::entry::DirectoryEntry, path::Path};

use {fiber_status::Status, midl_meshx_io as mio, std::sync::Arc};

/// Defines the type of the new entry to be created via the [`EntryConstructor::create_entry()`]
/// call.
///
/// It is set by certain flags in the `flags` argument of the `Directory::Open()` fuchsia.io call.
/// While it is possible to issue an `Open` call that will try to create, say, a "service" or a
/// "block device", these use cases are undefined at the moment.  So the library hides them from
/// the library users and will just return an error to the FIDL client.  Should we have a use case
/// where it would be possible to create a service or another kind of entry we should augment this
/// enumeration will a corresponding type.
#[derive(Debug, Copy, Clone, Eq, PartialEq)]
pub enum NewEntryType {
    Directory,
    File,
}

/// Should a file system support entry creation, it will need to provide an object implementing
/// this trait and register it in the [`crate::execution_scope::ExecutionScope`].  Connections will
/// use this trait when constructing new entries.
pub trait EntryConstructor {
    /// This method is called when a new entry need to be constructed.  The constructor is only
    /// responsibility is to create a new entry.  It does not need to attach the entry to the
    /// parent, nor does it need to establish connections to the new entry.  All of that is
    /// handled elsewhere.  But `parent` can be used to understand a position in the tree, if
    /// different kinds of entries need to be constructed in different parts of the file system
    /// tree.
    ///
    /// `parent` refers to the parent directory, where a new entry need to be added.
    ///
    /// `what` is the kind of entry that the user has requested.
    ///
    /// `name` is the name of the new entry relative to the `parent`.
    ///
    /// If a missing entry was encountered before reaching a leaf node, `path` will be a non-empty
    /// path that goes "inside" of the missing entry.  Common behaviour is that when this is
    /// non-empty a `NOT_FOUND` error is returned, as it is generally the case that only a leaf
    /// entry can be created.  But if a file system wants to allow creation of more than one
    /// element of a path, this argument allows for that.
    fn create_entry(
        self: Arc<Self>,
        parent: Arc<dyn DirectoryEntry>,
        what: NewEntryType,
        name: &str,
        path: &Path,
    ) -> Result<Arc<dyn DirectoryEntry>, Status>;
}

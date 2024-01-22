// Copyright 2019 The Fuchsia Authors. All rights reserved.
// Use of this source code is governed by a BSD-style license that can be
// found in the LICENSE file.

//! This is an implementation of "simple" pseudo directories.
//! Use [`mod@crate::directory::immutable::simple`]
//! to construct actual instances.  See [`Simple`] for details.

use std::{
    collections::BTreeMap,
    marker::PhantomData,
    sync::{Arc, Mutex},
};

use crate::name::Name;

use super::{entry::DirectoryEntry, connection::DerivedConnection};
 

/// An implementation of a "simple" pseudo directory.  This directory holds a set of entries,
/// allowing the server to add or remove entries via the
/// [`crate::directory::helper::DirectlyMutable::add_entry()`] and
/// [`crate::directory::helper::DirectlyMutable::remove_entry`] methods, and, depending on the
/// connection been used (see [`ImmutableConnection`] or [`MutableConnection`])
/// it may also allow the clients to modify the entries as well.  This is a common implementation
/// for [`mod@crate::directory::immutable::simple`] and [`mod@crate::directory::mutable::simple`].
pub struct Simple<Connection> {
    inner: Mutex<Inner>,

    // The inode for this directory. This should either be unique within this VFS, or INO_UNKNOWN.
    inode: u64,

    _connection: PhantomData<Connection>,

    not_found_handler: Mutex<Option<Box<dyn FnMut(&str) + Send + Sync + 'static>>>,
}

struct Inner {
    entries: BTreeMap<Name, Arc<dyn DirectoryEntry>>,

    // TODO: watchers: Watchers,
}

impl<Connection> Simple<Connection>
where
    Connection: DerivedConnection + 'static,
{
    pub(super) fn new(inode: u64) -> Arc<Self> {
        Arc::new(Simple {
            inner: Mutex::new(Inner { entries: BTreeMap::new() }),
            _connection: PhantomData,
            inode,
            not_found_handler: Mutex::new(None),
        })
    }
}
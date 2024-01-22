// Copyright 2019 The Fuchsia Authors. All rights reserved.
// Use of this source code is governed by a BSD-style license that can be
// found in the LICENSE file.

//! Values of this type represent "execution scopes" used by the library to give fine grained
//! control of the lifetimes of the tasks associated with particular connections.  When a new
//! connection is attached to a pseudo directory tree, an execution scope is provided.  This scope
//! is then used to start any tasks related to this connection.  All connections opened as a result
//! of operations on this first connection will also use the same scope, as well as any tasks
//! related to those connections.
//!
//! This way, it is possible to control the lifetime of a group of connections.  All connections
//! and their tasks can be shutdown by calling `shutdown` method on the scope that is hosting them.
//! Scope will also shutdown all the tasks when it goes out of scope.
//!
//! Implementation wise, execution scope is just a proxy, that forwards all the tasks to an actual
//! executor, provided as an instance of a [`futures::task::Spawn`] trait.

use crate::{directory::mutable::entry_constructor::EntryConstructor};

use {
    futures::{
        channel::oneshot,
        task::{self, Context, Poll, Waker},
        Future,
    },
     
    std::{
        ops::Drop,
        pin::Pin,
        sync::{Arc, Mutex},
    },
};

pub type SpawnError = task::SpawnError;

/// An execution scope that is hosting tasks for a group of connections.  See the module level
/// documentation for details.
///
/// Actual execution will be delegated to an "upstream" executor - something that implements
/// [`futures::task::Spawn`].  In a sense, this is somewhat of an analog of a multithreaded capable
/// [`futures::stream::FuturesUnordered`], but this some additional functionality specific to the
/// vfs library.
///
/// Use [`ExecutionScope::new()`] or [`ExecutionScope::build()`] to construct new
/// `ExecutionScope`es.
pub struct ExecutionScope {}

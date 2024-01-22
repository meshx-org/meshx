// Copyright 2019 The Fuchsia Authors. All rights reserved.
// Copyright 2024 MeshX Contributors. All rights reserved.

//! Tools for providing MeshX services.

mod service;

use anyhow::Error;
use fiber_rust as fx;
use std::sync::Arc;
use thiserror::Error;

use futures::{
    channel::mpsc::{self, UnboundedReceiver, UnboundedSender},
    future::BoxFuture,
    FutureExt,
};

use vfs::{
    directory::{
        // entry::DirectoryEntry,
        // helper::DirectlyMutable,
        immutable::{connection::ImmutableConnection, simple::simple},
        simple::Simple,
    },
    // execution_scope::ExecutionScope,
    // file::vmo::VmoFile,
    name::Name,
    path::Path,
    // remote::{remote_dir, remote_node},
    // service::endpoint,
};

use meshx_runtime::{take_startup_handle, HandleType};
use pin_project::pin_project;
use service::{ServiceObjLocal, ServiceObjTrait};

/// A filesystem which connects clients to services.
///
/// This type implements the `Stream` trait and will yield the values
/// returned from calling `Service::connect` on the services it hosts.
///
/// This can be used to, for example, yield streams of channels, request
/// streams, futures to run, or any other value that should be processed
/// as the result of a request.
#[must_use]
#[pin_project]
pub struct ServiceFs<ServiceObjTy: ServiceObjTrait> {
    // The root directory.
    dir: Arc<Simple<ImmutableConnection>>,

    // New connections are sent via an mpsc. The tuple is (index, channel) where index is the index
    // into the `services` member.
    new_connection_sender: UnboundedSender<(usize, fx::Channel)>,
    new_connection_receiver: UnboundedReceiver<(usize, fx::Channel)>,

    // A collection of objects that are able to handle new connections and convert them into a
    // stream of ServiceObjTy::Output requests.  There will be one for each service in the
    // filesystem (irrespective of its place in the hierarchy).
    services: Vec<ServiceObjTy>,

    // A future that completes when the VFS no longer has any connections.  These connections are
    // distinct from connections that might be to services or remotes within this filesystem.
    shutdown: BoxFuture<'static, ()>,

    // The filesystem does not start servicing any requests until ServiceFs is first polled.  This
    // preserves behaviour of ServiceFs from when it didn't use the Rust VFS, and is relied upon in
    // some cases.  The queue is used until first polled.  After that, `channel_queue` will be None
    // and requests to service channels will be actioned immediately (potentially on different
    // threads depending on the executor).
    channel_queue: Option<Vec<midl::endpoints::ServerEnd<mio::DirectoryMarker>>>,
}

/// A directory within a `ServiceFs`.
///
/// Services and subdirectories can be added to it.
pub struct ServiceFsDir<'a, ServiceObjTy: ServiceObjTrait> {
    fs: &'a mut ServiceFs<ServiceObjTy>,
    dir: Arc<Simple<ImmutableConnection>>,
}

// Not part of a trait so that clients won't have to import a trait
// in order to call these functions.
macro_rules! add_functions {
    () => {
        /// Returns a reference to the subdirectory at the given path,
        /// creating one if none exists.
        ///
        /// The path must be a single component.
        /// The path must be a valid `fuchsia.io` [`Name`].
        ///
        /// Panics if a service has already been added at the given path.
        pub fn dir(&mut self, path: impl Into<String>) -> ServiceFsDir<'_, ServiceObjTy> {
            let path: String = path.into();
            let name: Name = path.try_into().expect("Invalid path");
            let dir = Arc::downcast(self.dir.get_or_insert(name, new_simple_dir).into_any())
                .unwrap_or_else(|_| panic!("Not a directory"));
            ServiceFsDir { fs: self.fs(), dir }
        }
    };
}

impl<ServiceObjTy: ServiceObjTrait> ServiceFsDir<'_, ServiceObjTy> {
    fn fs(&mut self) -> &mut ServiceFs<ServiceObjTy> {
        self.fs
    }

    add_functions!();
}

impl<'a, Output: 'a> ServiceFs<ServiceObjLocal<'a, Output>> {
    /// Create a new `ServiceFs` that is singlethreaded-only and does not
    /// require services to implement `Send`.
    pub fn new_local() -> Self {
        Self::new_impl()
    }
}

fn new_simple_dir() -> Arc<Simple<ImmutableConnection>> {
    let dir = simple();

    dir.clone().set_not_found_handler(Box::new(move |path| {
        log::warn!(
            "ServiceFs received request to `{}` but has not been configured to serve this path.",
            path
        );
    }));

    dir
}

impl<ServiceObjTy: ServiceObjTrait> ServiceFs<ServiceObjTy> {
    fn new_impl() -> Self {
        let (new_connection_sender, new_connection_receiver) = mpsc::unbounded();
        //let scope = ExecutionScope::new();
        let dir = new_simple_dir();

        Self {
            //scope: scope.clone(),
            dir,
            new_connection_sender,
            new_connection_receiver,
            services: Vec::new(),
            shutdown: async move {}.boxed(),
            channel_queue: Some(Vec::new()),
        }
    }

    add_functions!();
}

/// An error indicating the startup handle on which the FIDL server
/// attempted to start was missing.
#[derive(Debug, Error)]
#[error("The startup handle on which the FIDL server attempted to start was missing.")]
pub struct MissingStartupHandle;

impl<ServiceObjTy: ServiceObjTrait> ServiceFs<ServiceObjTy> {
    /// Removes the `DirectoryRequest` startup handle for the current
    /// component and adds connects it to this `ServiceFs` as a client.
    ///
    /// Multiple calls to this function from the same component will
    /// result in `Err(MissingStartupHandle)`.
    pub fn take_and_serve_directory_handle(&mut self) -> Result<&mut Self, Error> {
        let startup_handle = take_startup_handle(HandleType::DirectoryRequest.into()).ok_or(MissingStartupHandle)?;

        self.serve_connection(midl::endpoints::ServerEnd::new(fx::Channel::from(startup_handle)))
    }

    /// Add a channel to serve this `ServiceFs` filesystem on. The `ServiceFs`
    /// will continue to be provided over previously added channels, including
    /// the one added if `take_and_serve_directory_handle` was called.
    pub fn serve_connection(
        &mut self,
        chan: midl::endpoints::ServerEnd<mio::DirectoryMarker>,
    ) -> Result<&mut Self, Error> {
        if let Some(channels) = &mut self.channel_queue {
            channels.push(chan);
        } else {
            self.serve_connection_impl(chan);
        }

        Ok(self)
    }
}

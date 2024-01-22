// Copyright 2019 The Fuchsia Authors. All rights reserved.
// Use of this source code is governed by a BSD-style license that can be
// found in the LICENSE file.

use crate::{
    directory::{
        // common::check_child_connection_flags,
        entry::DirectoryEntry,
        entry_container::Directory,
        mutable::entry_constructor::NewEntryType,
        // read_dirents,
        DirectoryOptions,
    },
    execution_scope::ExecutionScope,
    node::{Node as _, OpenNode},
    path::Path,
    traversal_position::TraversalPosition,
};

use {
    anyhow::Error,
    async_trait::async_trait,
    fiber_status::Status,
    futures::future::poll_fn,
    midl::endpoints::ServerEnd,
    midl_meshx_io as mio,
    std::{convert::TryInto as _, default::Default, sync::Arc, task::Poll},
    tracing::{span, Level},
};

/// Return type for `BaseConnection::handle_request` and [`DerivedConnection::handle_request`].
pub enum ConnectionState {
    /// Connection is still alive.
    Alive,
    /// Connection have received Node::Close message and should be closed.
    Closed,
}

/// This is an API a derived directory connection needs to implement, in order for the
/// `BaseConnection` to be able to interact with it.
pub trait DerivedConnection: Send + Sync {
    type Directory: Directory + ?Sized;

    /// Whether these connections support mutable connections.
    const MUTABLE: bool;

    /// Creates entry of the specified type `NewEntryType`.
    fn create_entry(
        scope: ExecutionScope,
        parent: Arc<dyn DirectoryEntry>,
        entry_type: NewEntryType,
        name: &str,
        path: &Path,
    ) -> Result<Arc<dyn DirectoryEntry>, Status>;
}

async fn yield_to_executor() {
    // Yield to the executor now, which should provide an opportunity for the spawned future to
    // run.
    let mut done = false;
    poll_fn(|cx| {
        if done {
            Poll::Ready(())
        } else {
            done = true;
            cx.waker().wake_by_ref();
            Poll::Pending
        }
    })
    .await;
}

/// Handles functionality shared between mutable and immutable FIDL connections to a directory.  A
/// single directory may contain multiple connections.  Instances of the `BaseConnection`
/// will also hold any state that is "per-connection".  Currently that would be the access flags
/// and the seek position.
pub(in crate::directory) struct BaseConnection<Connection>
where
    Connection: DerivedConnection + 'static,
{
    /// Execution scope this connection and any async operations and connections it creates will
    /// use.
    pub(in crate::directory) scope: ExecutionScope,

    pub(in crate::directory) directory: OpenNode<Connection::Directory>,

    /// Flags set on this connection when it was opened or cloned.
    pub(in crate::directory) options: DirectoryOptions,

    /// Seek position for this connection to the directory.  We just store the element that was
    /// returned last by ReadDirents for this connection.  Next call will look for the next element
    /// in alphabetical order and resume from there.
    ///
    /// An alternative is to use an intrusive tree to have a dual index in both names and IDs that
    /// are assigned to the entries in insertion order.  Then we can store an ID instead of the
    /// full entry name.  This is what the C++ version is doing currently.
    ///
    /// It should be possible to do the same intrusive dual-indexing using, for example,
    ///
    ///     https://docs.rs/intrusive-collections/0.7.6/intrusive_collections/
    ///
    /// but, as, I think, at least for the pseudo directories, this approach is fine, and it simple
    /// enough.
    seek: TraversalPosition,
}

impl<Connection> BaseConnection<Connection>
where
    Connection: DerivedConnection,
{
    /// Constructs an instance of `BaseConnection` - to be used by derived connections, when they
    /// need to create a nested `BaseConnection` "sub-object".  But when implementing
    /// `create_connection`, derived connections should use the [`create_connection`] call.
    pub(in crate::directory) fn new(
        scope: ExecutionScope,
        directory: OpenNode<Connection::Directory>,
        options: DirectoryOptions,
    ) -> Self {
        BaseConnection {
            scope,
            directory,
            options,
            seek: Default::default(),
        }
    }

    /// Handle a [`DirectoryRequest`].  This function is responsible for handing all the basic
    /// directory operations.
    pub(in crate::directory) async fn handle_request(
        &mut self,
        request: mio::DirectoryRequest,
    ) -> Result<ConnectionState, Error> {
        match request {
            mio::DirectoryRequest::Clone {
                flags,
                object,
                control_handle: _,
            } => {
                let span = span!(Level::TRACE, "Directory::Clone");
                let _ = span.enter();

                self.handle_clone(flags, object);
            }
            mio::DirectoryRequest::Reopen {
                rights_request: _,
                object_request,
                control_handle: _,
            } => {
                let span = span!(Level::TRACE, "Directory::Reopen");
                let _ = span.enter();
                // TODO(https://fxbug.dev/77623): Handle unimplemented io2 method.
                // Suppress any errors in the event a bad `object_request` channel was provided.
                let _: Result<_, _> = object_request.close_with_epitaph(Status::NOT_SUPPORTED);
            }
            mio::DirectoryRequest::Close { responder } => {
                let span = span!(Level::TRACE, "Directory::Reopen");
                let _ = span.enter();

                responder.send(Ok(()))?;
                return Ok(ConnectionState::Closed);
            }
            mio::DirectoryRequest::GetConnectionInfo { responder } => {
                trace::duration!("storage", "Directory::GetConnectionInfo");
                // TODO(https://fxbug.dev/77623): Restrict GET_ATTRIBUTES, ENUMERATE, and TRAVERSE.
                // TODO(https://fxbug.dev/77623): Implement MODIFY_DIRECTORY and UPDATE_ATTRIBUTES.
                responder.send(mio::ConnectionInfo {
                    rights: Some(self.options.rights),
                    ..Default::default()
                })?;
            }
            mio::DirectoryRequest::GetAttr { responder } => {
                async move {
                    let (attrs, status) = match self.directory.get_attrs().await {
                        Ok(attrs) => (attrs, Status::OK.into_raw()),
                        Err(status) => (
                            mio::NodeAttributes {
                                mode: 0,
                                id: mio::INO_UNKNOWN,
                                content_size: 0,
                                storage_size: 0,
                                link_count: 1,
                                creation_time: 0,
                                modification_time: 0,
                            },
                            status.into_raw(),
                        ),
                    };
                    responder.send(status, &attrs)
                }
                .trace(trace::trace_future_args!("storage", "Directory::GetAttr"))
                .await?;
            }
            mio::DirectoryRequest::GetAttributes { query, responder } => {
                async move {
                    let result = self.directory.get_attributes(query).await;
                    responder.send(
                        result
                            .as_ref()
                            .map(|a| {
                                let mio::NodeAttributes2 {
                                    mutable_attributes: m,
                                    immutable_attributes: i,
                                } = a;
                                (m, i)
                            })
                            .map_err(|status| Status::into_raw(*status)),
                    )
                }
                .trace(trace::trace_future_args!("storage", "Directory::GetAttributes"))
                .await?;
            }
            mio::DirectoryRequest::UpdateAttributes { payload: _, responder } => {
                trace::duration!("storage", "Directory::UpdateAttributes");
                // TODO(https://fxbug.dev/77623): Handle unimplemented io2 method.
                responder.send(Err(Status::NOT_SUPPORTED.into_raw()))?;
            }
            mio::DirectoryRequest::ListExtendedAttributes { iterator, .. } => {
                trace::duration!("storage", "Directory::ListExtendedAttributes");
                iterator.close_with_epitaph(Status::NOT_SUPPORTED)?;
            }
            mio::DirectoryRequest::GetExtendedAttribute { responder, .. } => {
                trace::duration!("storage", "Directory::GetExtendedAttribute");
                responder.send(Err(Status::NOT_SUPPORTED.into_raw()))?;
            }
            mio::DirectoryRequest::SetExtendedAttribute { responder, .. } => {
                trace::duration!("storage", "Directory::SetExtendedAttribute");
                responder.send(Err(Status::NOT_SUPPORTED.into_raw()))?;
            }
            mio::DirectoryRequest::RemoveExtendedAttribute { responder, .. } => {
                trace::duration!("storage", "Directory::RemoveExtendedAttribute");
                responder.send(Err(Status::NOT_SUPPORTED.into_raw()))?;
            }
            mio::DirectoryRequest::GetFlags { responder } => {
                trace::duration!("storage", "Directory::GetFlags");
                responder.send(Status::OK.into_raw(), self.options.to_io1())?;
            }
            mio::DirectoryRequest::SetFlags { flags: _, responder } => {
                trace::duration!("storage", "Directory::SetFlags");
                responder.send(Status::NOT_SUPPORTED.into_raw())?;
            }
            mio::DirectoryRequest::Open {
                flags,
                mode: _,
                path,
                object,
                control_handle: _,
            } => {
                {
                    trace::duration!("storage", "Directory::Open");
                    self.handle_open(flags, path, object);
                }
                // Since open typically spawns a task, yield to the executor now to give that task a
                // chance to run before we try and process the next request for this directory.
                yield_to_executor().await;
            }
            mio::DirectoryRequest::Open2 {
                path,
                mut protocols,
                object_request,
                control_handle: _,
            } => {
                {
                    trace::duration!("storage", "Directory::Open2");
                    // Fill in rights from the parent connection if it's absent.
                    if let mio::ConnectionProtocols::Node(mio::NodeOptions { rights, protocols, .. }) = &mut protocols {
                        if rights.is_none() {
                            if matches!(protocols, Some(mio::NodeProtocols { node: Some(_), .. })) {
                                // Only inherit the GET_ATTRIBUTES right for node connections.
                                *rights = Some(self.options.rights & mio::Operations::GET_ATTRIBUTES);
                            } else {
                                *rights = Some(self.options.rights);
                            }
                        }
                    }
                    // If optional_rights is set, remove any rights that are not present on the
                    // current connection.
                    if let mio::ConnectionProtocols::Node(mio::NodeOptions {
                        protocols:
                            Some(mio::NodeProtocols {
                                directory:
                                    Some(mio::DirectoryProtocolOptions {
                                        optional_rights: Some(rights),
                                        ..
                                    }),
                                ..
                            }),
                        ..
                    }) = &mut protocols
                    {
                        *rights &= self.options.rights;
                    }
                    protocols
                        .to_object_request(object_request)
                        .handle(|req| self.handle_open2(path, protocols, req));
                }
                // Since open typically spawns a task, yield to the executor now to give that task a
                // chance to run before we try and process the next request for this directory.
                yield_to_executor().await;
            }
            mio::DirectoryRequest::AdvisoryLock { request: _, responder } => {
                trace::duration!("storage", "Directory::AdvisoryLock");
                responder.send(Err(Status::NOT_SUPPORTED.into_raw()))?;
            }
            mio::DirectoryRequest::ReadDirents { max_bytes, responder } => {
                async move {
                    let (status, entries) = self.handle_read_dirents(max_bytes).await;
                    responder.send(status.into_raw(), entries.as_slice())
                }
                .trace(trace::trace_future_args!("storage", "Directory::ReadDirents"))
                .await?;
            }
            mio::DirectoryRequest::Enumerate {
                options: _,
                iterator,
                control_handle: _,
            } => {
                trace::duration!("storage", "Directory::Enumerate");
                // TODO(https://fxbug.dev/77623): Handle unimplemented io2 method.
                // Suppress any errors in the event a bad `iterator` channel was provided.
                let _ = iterator.close_with_epitaph(Status::NOT_SUPPORTED);
            }
            mio::DirectoryRequest::Rewind { responder } => {
                trace::duration!("storage", "Directory::Rewind");
                self.seek = Default::default();
                responder.send(Status::OK.into_raw())?;
            }
            mio::DirectoryRequest::Link {
                src,
                dst_parent_token,
                dst,
                responder,
            } => {
                async move {
                    let status: Status = self.handle_link(&src, dst_parent_token, dst).await.into();
                    responder.send(status.into_raw())
                }
                .trace(trace::trace_future_args!("storage", "Directory::Link"))
                .await?;
            }
            mio::DirectoryRequest::Watch {
                mask,
                options,
                watcher,
                responder,
            } => {
                trace::duration!("storage", "Directory::Watch");
                let status = if options != 0 {
                    Status::INVALID_ARGS
                } else {
                    let watcher = watcher.try_into()?;
                    self.handle_watch(mask, watcher).into()
                };
                responder.send(status.into_raw())?;
            }
            mio::DirectoryRequest::Query { responder } => {
                let () = responder.send(mio::DIRECTORY_PROTOCOL_NAME.as_bytes())?;
            }
            mio::DirectoryRequest::QueryFilesystem { responder } => {
                trace::duration!("storage", "Directory::QueryFilesystem");
                match self.directory.query_filesystem() {
                    Err(status) => responder.send(status.into_raw(), None)?,
                    Ok(info) => responder.send(0, Some(&info))?,
                }
            }
            mio::DirectoryRequest::Unlink {
                name: _,
                options: _,
                responder,
            } => {
                responder.send(Err(Status::NOT_SUPPORTED.into_raw()))?;
            }
            mio::DirectoryRequest::GetToken { responder } => {
                responder.send(Status::NOT_SUPPORTED.into_raw(), None)?;
            }
            mio::DirectoryRequest::Rename {
                src: _,
                dst_parent_token: _,
                dst: _,
                responder,
            } => {
                responder.send(Err(Status::NOT_SUPPORTED.into_raw()))?;
            }
            mio::DirectoryRequest::SetAttr {
                flags: _,
                attributes: _,
                responder,
            } => {
                responder.send(Status::NOT_SUPPORTED.into_raw())?;
            }
            mio::DirectoryRequest::Sync { responder } => {
                responder.send(Err(Status::NOT_SUPPORTED.into_raw()))?;
            }
            mio::DirectoryRequest::CreateSymlink { responder, .. } => {
                responder.send(Err(Status::NOT_SUPPORTED.into_raw()))?;
            }
        }
        Ok(ConnectionState::Alive)
    }

    /*fn handle_clone(&self, flags: mio::OpenFlags, server_end: ServerEnd<mio::NodeMarker>) {
        let describe = flags.intersects(mio::OpenFlags::DESCRIBE);
        let flags = match inherit_rights_for_clone(self.options.to_io1(), flags) {
            Ok(updated) => updated,
            Err(status) => {
                send_on_open_with_error(describe, server_end, status);
                return;
            }
        };

        self.directory.clone().open(self.scope.clone(), flags, Path::dot(), server_end);
    }

    fn handle_open(
        &self,
        mut flags: mio::OpenFlags,
        path: String,
        server_end: ServerEnd<mio::NodeMarker>,
    ) {
        let describe = flags.intersects(mio::OpenFlags::DESCRIBE);

        let path = match Path::validate_and_split(path) {
            Ok(path) => path,
            Err(status) => {
                send_on_open_with_error(describe, server_end, status);
                return;
            }
        };

        if path.is_dir() {
            flags |= mio::OpenFlags::DIRECTORY;
        }

        let flags = match check_child_connection_flags(self.options.to_io1(), flags) {
            Ok(updated) => updated,
            Err(status) => {
                send_on_open_with_error(describe, server_end, status);
                return;
            }
        };
        if path.is_dot() {
            if flags.intersects(mio::OpenFlags::NOT_DIRECTORY) {
                send_on_open_with_error(describe, server_end, Status::INVALID_ARGS);
                return;
            }
            if flags.intersects(mio::OpenFlags::CREATE_IF_ABSENT) {
                send_on_open_with_error(describe, server_end, Status::ALREADY_EXISTS);
                return;
            }
        }

        // It is up to the open method to handle OPEN_FLAG_DESCRIBE from this point on.
        let directory = self.directory.clone();
        directory.open(self.scope.clone(), flags, path, server_end);
    }

    fn handle_open2(
        &self,
        path: String,
        protocols: mio::ConnectionProtocols,
        object_request: ObjectRequestRef<'_>,
    ) -> Result<(), Status> {
        let path = Path::validate_and_split(path)?;

        if let Some(rights) = protocols.rights() {
            if rights.intersects(!self.options.rights) {
                return Err(Status::ACCESS_DENIED);
            }
        }

        // If requesting attributes, check permission.
        if !object_request.attributes().is_empty()
            && !self.options.rights.contains(mio::Operations::GET_ATTRIBUTES)
        {
            return Err(Status::ACCESS_DENIED);
        }

        // If creating an object, it's not legal to specify more than one protocol.
        //
        // TODO(b/293947862): If we add an additional node type, we will need to update this. See if
        // there is a more generic or robust way to check this so that we don't miss any node types.
        if protocols.open_mode() != mio::OpenMode::OpenExisting
            && ((protocols.is_file_allowed() && protocols.is_dir_allowed())
                || protocols.is_symlink_allowed())
        {
            return Err(Status::INVALID_ARGS);
        }

        if protocols.create_attributes().is_some()
            && protocols.open_mode() == mio::OpenMode::OpenExisting
        {
            return Err(Status::INVALID_ARGS);
        }

        if path.is_dot() {
            if !protocols.is_node() && !protocols.is_dir_allowed() {
                return Err(Status::INVALID_ARGS);
            }
            if protocols.open_mode() == mio::OpenMode::AlwaysCreate {
                return Err(Status::ALREADY_EXISTS);
            }
        }

        self.directory.clone().open2(self.scope.clone(), path, protocols, object_request)
    }

    async fn handle_read_dirents(&mut self, max_bytes: u64) -> (Status, Vec<u8>) {
        async {
            let (new_pos, sealed) =
                self.directory.read_dirents(&self.seek, read_dirents::Sink::new(max_bytes)).await?;
            self.seek = new_pos;
            let read_dirents::Done { buf, status } = *sealed
                .open()
                .downcast::<read_dirents::Done>()
                .map_err(|_: Box<dyn std::any::Any>| {
                    #[cfg(debug)]
                    panic!(
                        "`read_dirents()` returned a `dirents_sink::Sealed`
                        instance that is not an instance of the \
                        `read_dirents::Done`. This is a bug in the \
                        `read_dirents()` implementation."
                    );
                    Status::NOT_SUPPORTED
                })?;
            Ok((status, buf))
        }
        .await
        .unwrap_or_else(|status| (status, Vec::new()))
    }

    async fn handle_link(
        &self,
        source_name: &str,
        target_parent_token: midl::Handle,
        target_name: String,
    ) -> Result<(), Status> {
        if source_name.contains('/') || target_name.contains('/') {
            return Err(Status::INVALID_ARGS);
        }

        if !self.options.rights.contains(mio::W_STAR_DIR) {
            return Err(Status::BAD_HANDLE);
        }

        let (target_parent, _flags) = self
            .scope
            .token_registry()
            .get_owner(target_parent_token)?
            .ok_or(Err(Status::NOT_FOUND))?;

        target_parent.link(target_name, self.directory.clone().into_any(), source_name).await
    }

    fn handle_watch(
        &mut self,
        mask: mio::WatchMask,
        watcher: DirectoryWatcher,
    ) -> Result<(), Status> {
        let directory = self.directory.clone();
        directory.register_watcher(self.scope.clone(), mask, watcher)
    }*/
}

/*#[async_trait]
impl<T: DerivedConnection + 'static> Representation for BaseConnection<T> {
    type Protocol = mio::DirectoryMarker;

    async fn get_representation(
        &self,
        requested_attributes: mio::NodeAttributesQuery,
    ) -> Result<mio::Representation, Status> {
        Ok(mio::Representation::Directory(mio::DirectoryInfo {
            attributes: Some(self.directory.get_attributes(requested_attributes).await?),
            ..Default::default()
        }))
    }

    async fn node_info(&self) -> Result<mio::NodeInfoDeprecated, Status> {
        Ok(mio::NodeInfoDeprecated::Directory(mio::DirectoryObject { supports_io2: false }))
    }
}*/

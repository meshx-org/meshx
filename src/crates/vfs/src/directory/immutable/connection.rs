// Copyright 2019 The Fuchsia Authors. All rights reserved.
// Use of this source code is governed by a BSD-style license that can be
// found in the LICENSE file.

//! Connection to a directory that can not be modified by the client, no matter what permissions
//! the client has on the FIDL connection.

use crate::{
    directory::{
        connection::{BaseConnection, ConnectionState, DerivedConnection},
        entry::DirectoryEntry,
        entry_container,
        mutable::entry_constructor::NewEntryType,
    },
    execution_scope::ExecutionScope,
    path::Path,
   
};

use {
    fiber_status::Status,
    futures::TryStreamExt as _,
    midl_meshx_io as mio,
    std::{future::Future, sync::Arc},
};

pub struct ImmutableConnection {
    base: BaseConnection<Self>,
}

impl ImmutableConnection {
    async fn handle_requests(mut self, mut requests: mio::DirectoryRequestStream) {
        while let Ok(Some(request)) = requests.try_next().await {
            let Some(_guard) = self.base.scope.try_active_guard() else {
                break;
            };
            
            if !matches!(self.base.handle_request(request).await, Ok(ConnectionState::Alive)) {
                break;
            }
        }
    }    
}

impl DerivedConnection for ImmutableConnection {
    type Directory = dyn entry_container::Directory;
    const MUTABLE: bool = false;

    fn create_entry(
        _scope: ExecutionScope,
        _parent: Arc<dyn DirectoryEntry>,
        _entry_type: NewEntryType,
        _name: &str,
        _path: &Path,
    ) -> Result<Arc<dyn DirectoryEntry>, Status> {
        Err(Status::NOT_SUPPORTED)
    }
}

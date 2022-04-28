// Copyright 2022 MeshX Contributors. All rights reserved.

mod handles;
mod types;

use crate::handles::{Channel, Handle, Process, Realm};
use crate::types::{ISyscall, Status};

pub struct Kernel;

impl ISyscall for Kernel {
    fn handle_duplicate(handle: Handle) -> types::HandleResult {
        types::HandleResult {
            status: Status::Ok,
            handle: handle,
        }
    }

    fn handle_replace(handle: Handle, replacement: Handle) -> types::HandleResult {
        types::HandleResult {
            status: Status::Ok,
            handle: handle,
        }
    }

    fn handle_close(handle: Handle) -> types::Result {
        types::Result { status: Status::Ok }
    }

    fn channel_create(process: Process) -> types::HandlePairResult {
        types::HandlePairResult {
            status: Status::Ok,
            right: 0,
            left: 0,
        }
    }

    fn channel_write(channel: Channel, data: Vec<u8>, handles: Vec<Handle>) -> types::WriteResult {
        types::WriteResult { status: Status::Ok }
    }

    fn channel_write_etc(
        channel: Channel,
        data: Vec<u8>,
        dispositions: Vec<types::HandleDisposition>,
    ) -> types::WriteResult {
        types::WriteResult { status: Status::Ok }
    }

    fn channel_read(channel: Channel) -> types::ReadResult {
        types::ReadResult { status: Status::Ok }
    }

    fn channel_read_etc(channel: Channel) -> types::ReadEtcResult {
        types::ReadEtcResult { status: Status::Ok }
    }

    fn realm_create(parent: Realm) -> types::HandleResult {
        types::HandleResult {
            status: Status::Ok,
            handle: 0,
        }
    }

    fn process_create(parent: Realm, name: &str, program: Handle) -> types::HandleResult {
        types::HandleResult {
            status: Status::Ok,
            handle: 0,
        }
    }

    fn process_start(process: Process, bootstrap: Handle) -> types::Result {
        types::Result { status: Status::Ok }
    }
}

impl Kernel {
    pub fn new() -> Self {
        Self {}
    }
}

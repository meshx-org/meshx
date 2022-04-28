// Copyright 2022 MeshX Contributors. All rights reserved.

use bitflags::bitflags;

use crate::handles::{Handle, Realm, Channel, Process};

#[derive(Debug)]
pub(crate) enum Status {
    Ok = 0,
    ErrInternal = 1,
    ErrNotSupported = 2,
    ErrNoMemory = 3,
    ErrWrongType = 4,
    ErrBadState = 5,
    ErrInvalidArgs = 6,
    ErrBadHandle = 7,
    ErrBadSyscall = 8,
}

bitflags! {
    pub(crate) struct HandleRights: u32 {
        const NONE    = 0b00000000;
        const READ    = 0b00000001;
        const WRITE   = 0b00000010;
        const EXECUTE     = 0b00000100;
        const DEFAULT     = Self::READ.bits | Self::WRITE.bits;
    }
}

#[derive(Debug)]
pub(crate) enum HandleType {
    Handle = 0,
    ChannelHandle = 1,
    RealmHandle = 2,
    ProcessHandle = 3,
}

#[derive(Debug)]
pub(crate) struct Result {
    pub status: Status
}

#[derive(Debug)]
pub(crate) struct HandleResult {
    pub status: Status,
    pub handle: Handle,
}

#[derive(Debug)]
pub(crate) struct WriteResult {
    pub status: Status
}

#[derive(Debug)]
pub(crate) struct ReadResult {
    pub status: Status
}

#[derive(Debug)]
pub(crate) struct ReadEtcResult {
    pub status: Status
}

#[derive(Debug)]
pub(crate) struct HandlePairResult {
    pub status: Status,
    pub right: Handle,
    pub left: Handle,
}

#[derive(Debug)]
pub(crate) struct HandleDisposition {
    // operation: HandleOp,
    pub handle: Handle,
    pub r#type: HandleType,
    pub rights: HandleRights, // HandleRights enum
}

pub(crate) trait ISyscall {
    // Handle operations.
    fn handle_duplicate(handle: Handle) -> HandleResult;
    fn handle_replace(handle: Handle, replacement: Handle) -> HandleResult;
    fn handle_close(handle: Handle) -> self::Result;

    // Channel operations.
    fn channel_create(process: Process) -> HandlePairResult;
    fn channel_write(channel: Channel, data: Vec<u8>, handles: Vec<Handle>) -> WriteResult;
    fn channel_write_etc(
        channel: Channel,
        data: Vec<u8>,
        dispositions: Vec<HandleDisposition>,
    ) -> WriteResult;
    fn channel_read(channel: Channel) -> ReadResult;
    fn channel_read_etc(channel: Channel) -> ReadEtcResult;

    // Realm operations.
    fn realm_create(parent: Realm) -> HandleResult;

    // Process operations.
    fn process_create(parent: Realm, name: &str, program: Handle) -> HandleResult;
    fn process_start(process: Process, bootstrap: Handle) -> self::Result;
}

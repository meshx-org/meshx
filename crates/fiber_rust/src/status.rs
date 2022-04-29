// Copyright 2022 MeshX Contributors. All rights reserved.
// Copyright 2018 The Fuchsia Authors. All rights reserved.
// Use of this source code is governed by a BSD-style license that can be
// found in the LICENSE file.

//! Type-safe bindings for Zircon status.

use fiber_sys as sys;

use crate::assoc_values;

/// Status type indicating the result of a Fuchsia syscall.
///
/// This type is generally used to indicate the reason for an error.
/// While this type can contain `Status::OK` (`ZX_OK` in C land), elements of this type are
/// generally constructed using the `ok` method, which checks for `ZX_OK` and returns a
/// `Result<(), Status>` appropriately.
#[derive(Copy, Clone, Ord, PartialOrd, Eq, PartialEq, Hash)]
#[repr(transparent)]
pub struct Status(sys::fx_status_t);

impl Status {
    /// Returns `Ok(())` if the status was `OK`,
    /// otherwise returns `Err(status)`.
    pub fn ok(raw: sys::fx_status_t) -> Result<(), Status> {
        if raw == Status::OK.0 {
            Ok(())
        } else {
            Err(Status(raw))
        }
    }

    pub fn from_raw(raw: sys::fx_status_t) -> Self {
        Status(raw)
    }

    pub fn into_raw(self) -> sys::fx_status_t {
        self.0
    }
}

assoc_values!(Status, [
    OK                     = sys::FX_OK;
    INTERNAL               = sys::FX_ERR_INTERNAL;
    NOT_SUPPORTED          = sys::FX_ERR_NOT_SUPPORTED;
    NO_RESOURCES           = sys::FX_ERR_NO_RESOURCES;
    NO_MEMORY              = sys::FX_ERR_NO_MEMORY;
    INTERRUPTED_RETRY      = sys::FX_ERR_INTERRUPTED_RETRY;
    INVALID_ARGS           = sys::FX_ERR_INVALID_ARGS;
    BAD_HANDLE             = sys::FX_ERR_BAD_HANDLE;
    WRONG_TYPE             = sys::FX_ERR_WRONG_TYPE;
    BAD_SYSCALL            = sys::FX_ERR_BAD_SYSCALL;
    OUT_OF_RANGE           = sys::FX_ERR_OUT_OF_RANGE;
    BUFFER_TOO_SMALL       = sys::FX_ERR_BUFFER_TOO_SMALL;
    BAD_STATE              = sys::FX_ERR_BAD_STATE;
    TIMED_OUT              = sys::FX_ERR_TIMED_OUT;
    SHOULD_WAIT            = sys::FX_ERR_SHOULD_WAIT;
    CANCELED               = sys::FX_ERR_CANCELED;
    PEER_CLOSED            = sys::FX_ERR_PEER_CLOSED;
    NOT_FOUND              = sys::FX_ERR_NOT_FOUND;
    ALREADY_EXISTS         = sys::FX_ERR_ALREADY_EXISTS;
    ALREADY_BOUND          = sys::FX_ERR_ALREADY_BOUND;
    UNAVAILABLE            = sys::FX_ERR_UNAVAILABLE;
    ACCESS_DENIED          = sys::FX_ERR_ACCESS_DENIED;
    BAD_PATH               = sys::FX_ERR_BAD_PATH;
    NOT_DIR                = sys::FX_ERR_NOT_DIR;
    NOT_FILE               = sys::FX_ERR_NOT_FILE;
    FILE_BIG               = sys::FX_ERR_FILE_BIG;
    NO_SPACE               = sys::FX_ERR_NO_SPACE;
    NOT_EMPTY              = sys::FX_ERR_NOT_EMPTY;
    STOP                   = sys::FX_ERR_STOP;
    NEXT                   = sys::FX_ERR_NEXT;
    ASYNC                  = sys::FX_ERR_ASYNC;
    PROTOCOL_NOT_SUPPORTED = sys::FX_ERR_PROTOCOL_NOT_SUPPORTED;
    ADDRESS_UNREACHABLE    = sys::FX_ERR_ADDRESS_UNREACHABLE;
    ADDRESS_IN_USE         = sys::FX_ERR_ADDRESS_IN_USE;
]);

impl std::fmt::Display for Status {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self.assoc_const_name() {
            Some(name) => name.fmt(f),
            None => write!(f, "Unknown zircon status code: {}", self.0),
        }
    }
}

impl Status {
    pub fn from_result(res: Result<(), Self>) -> Self {
        res.into()
    }
}

impl From<Result<(), Status>> for Status {
    fn from(res: Result<(), Status>) -> Status {
        match res {
            Ok(()) => Self::OK,
            Err(status) => status,
        }
    }
}
impl From<Status> for Result<(), Status> {
    fn from(src: Status) -> Result<(), Status> {
        Status::ok(src.into_raw())
    }
}

#[cfg(test)]
mod test {
    use super::Status;
    #[test]
    fn status_debug_format() {
        let cases = [
            ("Status(OK)", Status::OK),
            ("Status(BAD_SYSCALL)", Status::BAD_SYSCALL),
            ("Status(NEXT)", Status::NEXT),
            ("Status(-5050)", Status(-5050)),
        ];
        for &(expected, value) in &cases {
            assert_eq!(expected, format!("{:?}", value));
        }
    }
    #[test]
    fn status_into_result() {
        let ok_result: Result<(), Status> = Status::OK.into();
        assert_eq!(ok_result, Ok(()));
        let err_result: Result<(), Status> = Status::BAD_SYSCALL.into();
        assert_eq!(err_result, Err(Status::BAD_SYSCALL));
    }
}

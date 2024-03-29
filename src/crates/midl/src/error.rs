// Copyright 2023 MeshX Contributors. All rights reserved.
// Copyright 2016 The Fuchsia Authors. All rights reserved.
// Use of this source code is governed by a BSD-style license that can be
// found in the LICENSE file.

//! Error (common to all fidl operations)

use {
    crate::handle::{ObjectType, Rights},
    fiber_status as fx_status,
    std::result,
    thiserror::Error,
};

/// A specialized `Result` type for FIDL operations.
pub type Result<T> = result::Result<T, Error>;

/// The error type used by FIDL operations.
#[derive(Error, Debug, Clone)]
#[non_exhaustive]
pub enum Error {
    /// Unexpected response to synchronous FIDL query.
    ///
    /// This will occur if an event or a response with an unexpected transaction
    /// ID is received when using the synchronous FIDL bindings.
    #[error("Unexpected response to synchronous FIDL query.")]
    UnexpectedSyncResponse,

    /// Invalid boolean
    #[error("Invalid FIDL boolean.")]
    InvalidBoolean,

    /// Invalid header for a FIDL buffer.
    #[error("Invalid header for a FIDL buffer.")]
    InvalidHeader,

    /// Unsupported wire format.
    #[error("Incompatible wire format magic number: {0}.")]
    IncompatibleMagicNumber(u8),

    /// Invalid FIDL buffer.
    #[error("Invalid FIDL buffer.")]
    Invalid,

    /// The FIDL object could not fit within the provided buffer range.
    #[error("The FIDL object could not fit within the provided buffer range")]
    OutOfRange,

    /// FIDL types bounded to less than or equal to 65520 bytes cannot be large messages.
    #[error("FIDL types bounded to less than or equal to 65520 bytes cannot be large messages.")]
    LargeMessageImpossible,

    // TODO(fxbug.dev/117162): There is a tracking bug keeping tabs on the eventual removal of this
    // limitation.
    /// Large FIDL messages must have <=63 handles, rather than the usual limit of 64.
    #[error("Large FIDL messages must have <=63 handles, rather than the usual limit of 64.")]
    LargeMessage64Handles,

    /// Large FIDL messages must have at least 1 handle pointing to the overflow VMO.
    #[error("Large FIDL messages must have at least 1 handle pointing to the overflow VMO.")]
    LargeMessageMissingHandles,

    /// Large FIDL messages must have properly formed overflow buffer handles.
    #[error("Large FIDL messages must have properly formed overflow buffer handles.")]
    LargeMessageInvalidOverflowBufferHandle,

    /// Large FIDL messages must have a well-formed 16-byte info struct
    #[error("Large FIDL messages must have a well-formed 16-byte info struct.")]
    LargeMessageInfoMissized {
        /// Observed size of the `LargeMessageInfo` struct.
        size: usize,
    },

    /// Large FIDL messages must have a properly formed info struct.
    #[error("Large FIDL messages must have a properly formed info struct.")]
    LargeMessageInfoMalformed,

    /// Large FIDL messages must be greater than 65520 bytes.
    #[error("Large FIDL messages must be greater than 65520 bytes.")]
    LargeMessageTooSmall {
        /// Observed size in the `LargeMessageInfo` struct.
        size: usize,
    },

    /// Writing the overflow VMO failed.
    #[error("Could not write the overflow VMO, due to status: {status}.")]
    LargeMessageCouldNotWriteVmo {
        /// Status returned when either when creating the VMO, or writing to it.
        status: fx_status::Status,
    },

    /// Reading the overflow VMO failed.
    #[error("Could not read the overflow VMO, due to status: {status}.")]
    LargeMessageCouldNotReadVmo {
        /// Status returned when either getting the VMO size, or reading from it.
        status: fx_status::Status,
    },
    /// Decoding the FIDL object did not use all of the bytes provided.
    #[error("Decoding the FIDL object did not use all of the bytes provided.")]
    ExtraBytes,

    /// Decoding the FIDL object did not use all of the handles provided.
    #[error("Decoding the FIDL object did not use all of the handles provided.")]
    ExtraHandles,

    /// Decoding the FIDL object observed non-zero value in a padding byte.
    #[error(
        "Decoding the FIDL object observed non-zero value in the padding region starting at byte {padding_start}."
    )]
    NonZeroPadding {
        /// Index of the first byte of the padding, relative to the beginning of the message.
        padding_start: usize,
    },

    /// The FIDL object had too many layers of out-of-line recursion.
    #[error("The FIDL object had too many layers of out-of-line recursion.")]
    MaxRecursionDepth,

    /// There was an attempt read or write a null-valued object as a non-nullable type.
    #[error("There was an attempt to read or write a null-valued object as a non-nullable FIDL type.")]
    NotNullable,

    /// A FIDL object reference with nonzero byte length had a null data pointer.
    #[error("A FIDL object reference with nonzero byte length had a null data pointer.")]
    UnexpectedNullRef,

    /// Incorrectly encoded UTF8.
    #[error("A FIDL message contained incorrectly encoded UTF8.")]
    Utf8Error,

    /// Exceeded the maximum length of a vector.
    #[error("Vector was too long. Expected at most {max_length} elements, got {actual_length}.")]
    VectorTooLong {
        /// Maximum length, i.e. the `N` in `vector<T>:N`.
        max_length: usize,
        /// Actual length of the vector (number of elements).
        actual_length: usize,
    },

    /// Exceeded the maximum length of a string.
    #[error("String was too long. Expected at most {max_bytes} bytes, got {actual_bytes}.")]
    StringTooLong {
        /// Maximum length in bytes, i.e. the `N` in `string:N`.
        max_bytes: usize,
        /// Actual length of the string in bytes.
        actual_bytes: usize,
    },

    /// A message was received for an ordinal value that the service does not
    /// understand, and either the method is not flexible or this protocol does
    /// not allow flexible methods of this type.  This generally results from an
    /// attempt to call a FIDL service of a type other than the one being
    /// served.
    #[error(
        "A message was received for ordinal value {ordinal} \
                   that the FIDL service {protocol_name} does not understand."
    )]
    UnknownOrdinal {
        /// The unknown ordinal.
        ordinal: u64,
        /// The name of the service for which the message was intended.
        protocol_name: &'static str,
    },

    /// A flexible method call was not recognized by the server.
    #[error("Server for the FIDL protocol {protocol_name} did not recognize method {method_name}.")]
    UnsupportedMethod {
        /// Name of the method that was called.
        method_name: &'static str,
        /// Name of the service for which the message was intended.
        protocol_name: &'static str,
    },

    /// Invalid bits value for a strict bits type.
    #[error("Invalid bits value for a strict bits type.")]
    InvalidBitsValue,

    /// Invalid enum value for a strict enum type.
    #[error("Invalid enum value for a strict enum type.")]
    InvalidEnumValue,

    /// Unrecognized descriminant for a FIDL union type.
    #[error("Unrecognized descriminant for a FIDL union type.")]
    UnknownUnionTag,

    /// A future was polled after it had already completed.
    #[error("A FIDL future was polled after it had already completed.")]
    PollAfterCompletion,

    /// A response message was received with txid 0.
    #[error("Invalid response with txid 0.")]
    InvalidResponseTxid,

    /// A presence indicator (for out-of-line data or a handle) had a value
    /// other than all 0x00 or all 0xFF.
    #[error("Invalid presence indicator.")]
    InvalidPresenceIndicator,

    /// An inline bit is unset when the type size is known to be <= 4 bytes.
    #[error("Invalid inline bit in envelope.")]
    InvalidInlineBitInEnvelope,

    /// An envelope inline marker is malformed.
    #[error("Invalid inline marker in envelope.")]
    InvalidInlineMarkerInEnvelope,

    /// An envelope in the FIDL message had an unexpected number of bytes.
    #[error("Invalid number of bytes in FIDL envelope.")]
    InvalidNumBytesInEnvelope,

    /// An envelope in the FIDL message had an unexpected number of handles.
    #[error("Invalid number of handles in FIDL envelope.")]
    InvalidNumHandlesInEnvelope,

    /// A handle which is invalid in the context of a host build of Fuchsia.
    #[error("Invalid FIDL handle used on the host.")]
    InvalidHostHandle,

    /// The handle subtype doesn't match the expected subtype.
    #[error("Incorrect handle subtype. Expected {}, but received {}", .expected.into_raw(), .received.into_raw())]
    IncorrectHandleSubtype {
        /// The expected object type.
        expected: ObjectType,
        /// The received object type.
        received: ObjectType,
    },

    /// Some expected handle rights are missing.
    #[error("Some expected handle rights are missing: {}", .missing_rights.bits())]
    MissingExpectedHandleRights {
        /// The rights that are missing.
        missing_rights: Rights,
    },

    /// A non-resource type encountered a handle when decoding unknown data
    #[error("Attempted to decode unknown data with handles for a non-resource type")]
    CannotStoreUnknownHandles,

    /// An error was encountered during handle replace().
    #[error("An error was encountered during handle replace()")]
    HandleReplace(#[source] fx_status::Status),

    /// A FIDL server encountered an IO error writing a response to a channel.
    #[error("A server encountered an IO error writing a FIDL response to a channel: {0}")]
    ServerResponseWrite(#[source] fx_status::Status),

    /// A FIDL server encountered an IO error reading incoming requests from a channel.
    #[error("A FIDL server encountered an IO error reading incoming FIDL requests from a channel: {0}")]
    ServerRequestRead(#[source] fx_status::Status),

    /// A FIDL server encountered an IO error writing an epitaph to a channel.
    #[error("A FIDL server encountered an IO error writing an epitaph into a channel: {0}")]
    ServerEpitaphWrite(#[source] fx_status::Status),

    /// A FIDL client encountered an IO error reading a response from a channel. For the
    /// `fx_status::Status::PEER_CLOSED` error, `Error::ClientChannelClosed` is used instead.
    #[error("A FIDL client encountered an IO error reading a response from a channel: {0}")]
    ClientRead(#[source] fx_status::Status),

    /// A FIDL client encountered an IO error writing a request to a channel. For the
    /// `fx_status::Status::PEER_CLOSED` error, `Error::ClientChannelClosed` is used instead.
    #[error("A FIDL client encountered an IO error writing a request into a channel: {0}")]
    ClientWrite(#[source] fx_status::Status),

    /// A FIDL client encountered an IO error issuing a channel call. For the
    /// `fx_status::Status::PEER_CLOSED` error, `Error::ClientChannelClosed` is used instead.
    #[error("A FIDL client encountered an IO error issuing a channel call: {0}")]
    ClientCall(#[source] fx_status::Status),

    /// A FIDL client encountered an IO error while waiting for an event. For the
    /// `fx_status::Status::PEER_CLOSED` error, `Error::ClientChannelClosed` is used instead.
    #[error("A FIDL client encountered an IO error issuing a channel call: {0}")]
    ClientEvent(#[source] fx_status::Status),

    /// A FIDL client's channel was closed. Contains an epitaph if one was sent by the server, or
    /// `fx_status::Status::PEER_CLOSED` otherwise.
    #[error("A FIDL client's channel to the service {protocol_name} was closed: {status}")]
    ClientChannelClosed {
        /// The epitaph or `fx_status::Status::PEER_CLOSED`.
        #[source]
        status: fx_status::Status,
        /// The name of the service at the other end of the channel.
        protocol_name: &'static str,
    },

    /// There was an error attaching a FIDL channel to the async executor.
    #[error("There was an error attaching a FIDL channel to the async executor: {0}")]
    AsyncChannel(#[source] fx_status::Status),

    /// There was a miscellaneous io::Error during a test.
    #[cfg(target_os = "fuchsia")]
    #[cfg(test)]
    #[error("Test fx_status::Status: {0}")]
    TestIo(#[source] fx_status::Status),
}

impl Error {
    /// Returns `true` if the error was sourced by a closed channel.
    pub fn is_closed(&self) -> bool {
        // The FIDL bindings should never report PEER_CLOSED errors via
        // ClientWrite or ClientRead; it should always be ClientChannelClosed.
        // But we keep these two checks in case old code relies on it.
        match self {
            Error::ClientRead(fx_status::Status::PEER_CLOSED)
            | Error::ClientWrite(fx_status::Status::PEER_CLOSED)
            | Error::ClientChannelClosed { .. }
            | Error::ServerRequestRead(fx_status::Status::PEER_CLOSED)
            | Error::ServerResponseWrite(fx_status::Status::PEER_CLOSED)
            | Error::ServerEpitaphWrite(fx_status::Status::PEER_CLOSED) => true,
            _ => false,
        }
    }
}

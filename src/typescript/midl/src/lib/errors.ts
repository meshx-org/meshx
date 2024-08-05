// Copyright 2023 The MeshX Authors. All rights reserved.
// Copyright 2022 The Fuchsia Authors. All rights reserved.
// Use of this source code is governed by a BSD-style license that can be
// found in the LICENSE file.

export enum ErrorCode {
    Unknown,
    ExceededSafeIntegerLimit,
    ExceededMaxOutOfLineDepth,
    InvalidBoolean,
    InvalidPresenceIndicator,
    InvalidNumBytesInEnvelope,
    InvalidNumHandlesInEnvelope,
    InvalidInlineMarkerInEnvelope,
    LargeMessage64Handles,
    LargeMessageImpossible,
    LargeMessageInfoMissized,
    LargeMessageInfoMalformed,
    LargeMessageInvalidOverflowBufferHandle,
    LargeMessageMissingHandles,
    LargeMessageTooSmall,
    TooFewBytes,
    TooManyBytes,
    TooFewHandles,
    TooManyHandles,
    StringTooLong,
    NonNullableTypeWithNullValue,
    StrictUnionUnknownField,
    UnknownMagic,
    InvalidBit,
    InvalidEnumValue,
    IntOutOfRange,
    NonEmptyStringWithNullBody,
    NonEmptyVectorWithNullBody,
    NonResourceHandle,
    MissingRequiredHandleRights,
    IncorrectHandleType,
    InvalidInlineBitInEnvelope,
    CountExceedsLimit,
    InvalidPaddingByte,
    UnknownMethod,
    UnsupportedWireFormat,
}

export class MidlError {
    constructor(public message: string, public code: ErrorCode = ErrorCode.Unknown) {}
}

export class MidlIntOutOfRangeError extends MidlError {
    constructor(value: number, min: number, max: number) {
        super(`IntOutOfRange: ${value} < ${min} or ${value} > ${max}`, ErrorCode.IntOutOfRange)
    }
}

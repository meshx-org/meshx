// Copyright 2023 The MeshX Authors. All rights reserved.
// Copyright 2022 The Fuchsia Authors. All rights reserved.
// Use of this source code is governed by a BSD-style license that can be
// found in the LICENSE file.

export enum FidlErrorCode {
    unknown,
    fidlExceededMaxOutOfLineDepth,
    fidlInvalidBoolean,
    fidlInvalidPresenceIndicator,
    fidlInvalidNumBytesInEnvelope,
    fidlInvalidNumHandlesInEnvelope,
    fidlInvalidInlineMarkerInEnvelope,
    fidlLargeMessage64Handles,
    fidlLargeMessageImpossible,
    fidlLargeMessageInfoMissized,
    fidlLargeMessageInfoMalformed,
    fidlLargeMessageInvalidOverflowBufferHandle,
    fidlLargeMessageMissingHandles,
    fidlLargeMessageTooSmall,
    fidlTooFewBytes,
    fidlTooManyBytes,
    fidlTooFewHandles,
    fidlTooManyHandles,
    fidlStringTooLong,
    fidlNonNullableTypeWithNullValue,
    fidlStrictUnionUnknownField,
    fidlUnknownMagic,
    fidlInvalidBit,
    fidlInvalidEnumValue,
    fidlIntOutOfRange,
    fidlNonEmptyStringWithNullBody,
    fidlNonEmptyVectorWithNullBody,
    fidlNonResourceHandle,
    fidlMissingRequiredHandleRights,
    fidlIncorrectHandleType,
    fidlInvalidInlineBitInEnvelope,
    fidlCountExceedsLimit,
    fidlInvalidPaddingByte,
    fidlUnknownMethod,
    fidlUnsupportedWireFormat,
}

export class FidlError {
    constructor(message: string, code: FidlErrorCode = FidlErrorCode.unknown) {}
}

export class FidlRangeCheckError extends FidlError {
    constructor(value: number, min: number, max: number) {
        super(`FidlRangeCheckError: ${value} < ${min} or ${value} > ${max}`, FidlErrorCode.fidlIntOutOfRange)
    }
}

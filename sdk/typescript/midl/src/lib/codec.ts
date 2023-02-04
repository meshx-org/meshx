// Copyright 2023 The MeshX Authors. All rights reserved.
// Copyright 2022 The Fuchsia Authors. All rights reserved.
// Use of this source code is governed by a BSD-style license that can be
// found in the LICENSE file.

import * as FX from '@meshx-org/fiber'

import { FidlError, FidlErrorCode, FidlRangeCheckError } from './errors'
import {
    CallStrictness,
    IncomingMessage,
    kMagicNumberInitial,
    kMessageDynamicFlagOffset,
    kMessageFlagOffset,
    kMessageHeaderSize,
    kMessageMagicOffset,
    kMessageOrdinalOffset,
    kMessageTxidOffset,
    kWireFormatV2FlagMask,
    OutgoingMessage,
    strictnessToFlags,
} from './message'
import { kEnvelopeInlineMarker, WireFormat } from './wireformat'

const _kAlignment = 8
const _kAlignmentMask = _kAlignment - 1
const _maxOutOfLineDepth = 32
const _kInitialBufferSize = 512
const _kMinBufferSizeIncreaseFactor = 2

export const align = (size: number) => (size + _kAlignmentMask) & ~_kAlignmentMask

function _checkRange(value: number, min: number, max: number): void {
    if (value < min || value > max) {
        throw new FidlRangeCheckError(value, min, max)
    }
}

export class Encoder {
    public data = new DataView(new ArrayBuffer(_kInitialBufferSize))

    private handleDispositions: Array<FX.HandleDisposition> = []
    private extent = 0
    private wireFormat: WireFormat

    constructor(wireFormat: WireFormat) {
        this.wireFormat = wireFormat
    }

    get message(): OutgoingMessage {
        this.encodeUint8(kWireFormatV2FlagMask, kMessageFlagOffset)
        const trimmed = this.data.buffer.slice(0, this.extent)
        return new OutgoingMessage(new DataView(trimmed), this.handleDispositions)
    }

    private grow(newSize: number): void {
        const newList = new Uint8Array(newSize)
        newList.set(new Uint8Array(this.data.buffer))
        this.data = new DataView(newList.buffer)
    }

    private claimBytes(claimSize: number): void {
        this.extent += claimSize
        if (this.extent > this.data.byteLength) {
            const newSize = Math.max(this.extent, _kMinBufferSizeIncreaseFactor * this.data.byteLength)
            this.grow(newSize)
        }
    }

    alloc(size: number, nextOutOfLineDepth: number): number {
        if (nextOutOfLineDepth > _maxOutOfLineDepth) {
            throw new FidlError('Exceeded maxOutOfLineDepth', FidlErrorCode.fidlExceededMaxOutOfLineDepth)
        }
        const offset = this.extent
        this.claimBytes(align(size))
        return offset
    }

    nextOffset(): number {
        return this.extent
    }

    countHandles(): number {
        return this.handleDispositions.length
    }

    addHandleDisposition(value: FX.HandleDisposition): void {
        this.handleDispositions.push(value)
    }

    encodeMessageHeader(ordinal: bigint, txid: number, strictness: CallStrictness): void {
        this.alloc(kMessageHeaderSize, 0)
        this.encodeUint32(txid, kMessageTxidOffset)
        this.encodeUint8(kWireFormatV2FlagMask, kMessageFlagOffset)
        this.encodeUint8(0, kMessageFlagOffset + 1)
        this.encodeUint8(strictnessToFlags(strictness), kMessageDynamicFlagOffset)
        this.encodeUint8(kMagicNumberInitial, kMessageMagicOffset)
        this.encodeUint64(ordinal, kMessageOrdinalOffset)
    }

    /// Produces a response for an UnknownMethod which was called with the given
    /// ordinal value for the method and the given transaction ID. This produces
    /// both a message header and an encoded body. The header will have the
    /// provided ordinal and txid and CallStrictness.flexible (this should never
    /// be used with a strict method). The body will contain a union with ordinal
    /// 3 and a value of type zx_status with the NOT_SUPPORTED error code.
    encodeUnknownMethodResponse(methodOrdinal: bigint, txid: number): void {
        this.encodeMessageHeader(methodOrdinal, txid, CallStrictness.flexible)
        const kUnknownMethodInlineSize = 16
        const kEnvelopeOffset = kMessageHeaderSize + 8
        const kTransportErrOrdinal = 3n

        this.alloc(kUnknownMethodInlineSize, 0)

        // Union header.
        // transport_err value for the union's ordinal.
        this.encodeUint64(kTransportErrOrdinal, kMessageHeaderSize)

        // Inline value of the zx_status.
        this.encodeInt32(FX.ERR_NOT_SUPPORTED, kEnvelopeOffset)

        // Number of handles in the envelope.
        this.encodeUint16(0, kEnvelopeOffset + 4)

        // Flags field, with tag indicating the value is stored in-line (in what
        // would otherwise be the size field).
        this.encodeUint16(kEnvelopeInlineMarker, kEnvelopeOffset + 6)
    }

    encodeBool(value: boolean, offset: number): void {
        this.data.setInt8(offset, value ? 1 : 0)
    }

    encodeInt8(value: number, offset: number): void {
        _checkRange(value, -128, 127)
        this.data.setInt8(offset, value)
    }

    encodeUint8(value: number, offset: number): void {
        _checkRange(value, 0, 255)
        this.data.setUint8(offset, value)
    }

    encodeInt16(value: number, offset: number): void {
        _checkRange(value, -32768, 32767)
        this.data.setInt16(offset, value, true)
    }

    encodeUint16(value: number, offset: number): void {
        _checkRange(value, 0, 65535)
        this.data.setUint16(offset, value, true)
    }

    encodeInt32(value: number, offset: number): void {
        _checkRange(value, -2147483648, 2147483647)
        this.data.setInt32(offset, value, true)
    }

    encodeUint32(value: number, offset: number): void {
        _checkRange(value, 0, 4294967295)
        this.data.setUint32(offset, value, true)
    }

    encodeInt64(value: bigint, offset: number): void {
        this.data.setBigInt64(offset, value, true)
    }

    encodeUint64(value: bigint, offset: number): void {
        this.data.setBigUint64(offset, value, true)
    }

    encodeFloat32(value: number, offset: number): void {
        this.data.setFloat32(offset, value, true)
    }

    encodeFloat64(value: number, offset: number): void {
        this.data.setFloat64(offset, value, true)
    }
}

export class Decoder {
    public data: DataView
    public handleInfos: FX.HandleInfo[]
    public wireFormat: WireFormat

    private nextOffset = 0
    private nextHandle = 0

    static fromMessage(message: IncomingMessage) {
        return new Decoder(message.data, message.handleInfos, message.parseWireFormat())
    }

    constructor(data: DataView, handleInfos: FX.HandleInfo[], wireFormat: WireFormat) {
        this.data = data
        this.handleInfos = handleInfos
        this.wireFormat = wireFormat
    }

    public claimBytes(size: number, nextOutOfLineDepth: number): number {
        if (nextOutOfLineDepth > _maxOutOfLineDepth) {
            throw new FidlError('Exceeded maxOutOfLineDepth', FidlErrorCode.fidlExceededMaxOutOfLineDepth)
        }
        const result = this.nextOffset
        this.nextOffset += align(size)
        if (this.nextOffset > this.data.byteLength) {
            throw new FidlError('Cannot access out of range memory', FidlErrorCode.fidlTooFewBytes)
        }
        return result
    }

    countUnclaimedBytes(): number {
        return this.data.byteLength - this.nextOffset
    }

    countClaimedHandles(): number {
        return this.nextHandle
    }

    countUnclaimedHandles(): number {
        return this.handleInfos.length - this.nextHandle
    }

    claimHandle(): FX.HandleInfo {
        if (this.nextHandle >= this.handleInfos.length) {
            throw new FidlError('Cannot access out of range handle', FidlErrorCode.fidlTooFewHandles)
        }
        return this.handleInfos[this.nextHandle++]
    }

    decodeBool(offset: number): boolean {
        switch (this.data.getUint8(offset)) {
            case 0:
                return false
            case 1:
                return true
            default:
                throw new FidlError('Invalid boolean', FidlErrorCode.fidlInvalidBoolean)
        }
    }

    decodeInt8 = (offset: number) => this.data.getInt8(offset)
    decodeUint8 = (offset: number) => this.data.getUint8(offset)
    decodeInt16 = (offset: number) => this.data.getInt16(offset, true)
    decodeUint16 = (offset: number) => this.data.getUint16(offset, true)
    decodeInt32 = (offset: number) => this.data.getInt32(offset, true)
    decodeUint32 = (offset: number) => this.data.getUint32(offset, true)
    decodeInt64 = (offset: number) => this.data.getBigInt64(offset, true)
    decodeUint64 = (offset: number) => this.data.getBigUint64(offset, true)
    decodeFloat32 = (offset: number) => this.data.getFloat32(offset, true)
    decodeFloat64 = (offset: number) => this.data.getFloat64(offset, true)

    checkPadding(offset: number, padding: number): void {
        for (let readAt = offset; readAt < offset + padding; readAt++) {
            if (this.data.getUint8(readAt) != 0) {
                throw new FidlError('Non-zero padding at: $readAt', FidlErrorCode.fidlInvalidPaddingByte)
            }
        }
    }
}

// Copyright 2023 The MeshX Authors. All rights reserved.
// Copyright 2022 The Fuchsia Authors. All rights reserved.
// Use of this source code is governed by a BSD-style license that can be
// found in the LICENSE file.

import * as FX from '@meshx-org/fiber'
import { align, Decoder, Encoder } from './codec'
import { FidlError, FidlErrorCode } from './errors'
import { WireFormat } from './wireformat'

export const kMessageHeaderSize = 16
export const kLargeMessageInfoSize = 16
export const kLargeMessageVmoRights =
    FX.RIGHT_GET_PROPERTY | FX.RIGHT_INSPECT | FX.RIGHT_READ | FX.RIGHT_TRANSFER | FX.RIGHT_WAIT

export const kMessageTxidOffset = 0
export const kMessageFlagOffset = 4
export const kMessageDynamicFlagOffset = 6
export const kMessageMagicOffset = 7
export const kMessageOrdinalOffset = 8
export const kMagicNumberInitial = 1
export const kWireFormatV2FlagMask = 2
const _kDynamicFlagsFlexible = 0x80
const _kDynamicFlagsByteOverflow = 0x40

export enum CallStrictness {
    strict,
    flexible,
}

enum CallOverflowing {
    large,
    small,
}

/** Convert CallStrictness to a byte that can be inserted into the dynamic flags portion of a message. */
export function strictnessToFlags(strictness: CallStrictness) {
    switch (strictness) {
        case CallStrictness.strict:
            return 0x00
        case CallStrictness.flexible:
            return _kDynamicFlagsFlexible
    }
}

/** Extract the CallStrictness from the dynamic flags byte of the message header. */
function strictnessFromFlags(dynamicFlags: number): CallStrictness {
    if ((dynamicFlags & _kDynamicFlagsFlexible) != 0) {
        return CallStrictness.flexible
    }
    return CallStrictness.strict
}

/** Extract the CallOverflowing from the dynamic flags byte of the message header. */
function overflowingFromFlags(dynamicFlags: number): CallOverflowing {
    if ((dynamicFlags & _kDynamicFlagsByteOverflow) != 0) {
        return CallOverflowing.large
    }
    return CallOverflowing.small
}

class BaseMessage {
    constructor(public readonly data: DataView) {}

    get txid() {
        return this.data.getUint32(kMessageTxidOffset, true)
    }

    get ordinal() {
        return this.data.getBigUint64(kMessageOrdinalOffset, true)
    }

    get magic() {
        return this.data.getUint8(kMessageMagicOffset)
    }

    parseWireFormat(): WireFormat {
        if ((this.data.getUint8(kMessageFlagOffset) & kWireFormatV2FlagMask) != 0) {
            return WireFormat.v2
        }

        throw new FidlError('unknown wire format', FidlErrorCode.fidlUnsupportedWireFormat)
    }

    get strictness(): CallStrictness {
        return strictnessFromFlags(this.data.getUint8(kMessageDynamicFlagOffset))
    }

    get overflowing(): CallOverflowing {
        return overflowingFromFlags(this.data.getUint8(kMessageDynamicFlagOffset))
    }

    isCompatible(): boolean {
        return this.magic == kMagicNumberInitial
    }

    hexDump(): void {
        const width = 16
        const list = new Uint8Array(this.data.buffer, 0)
        const buffer: string[] = []
        const isPrintable = /r'\w'/

        for (let i = 0; i < this.data.byteLength; i += width) {
            const hex: string[] = []
            const printable: string[] = []

            for (let j = 0; j < width && i + j < this.data.byteLength; j++) {
                const v = list[i + j]
                let s = v.toString(16)

                if (s.length == 1) {
                    hex.push(`0${s} `)
                } else {
                    hex.push(`${s} `)
                }

                s = String.fromCharCode(v)

                if (isPrintable.test(s)) {
                    printable.push(s)
                } else {
                    printable.push('.')
                }
            }

            buffer.push(`${hex.toString().padEnd(3 * width)} ${printable}\n`)
        }

        console.log(
            '==================================================\n' +
                buffer +
                '=================================================='
        )
    }
}

export class IncomingMessage extends BaseMessage {
    public handleInfos: FX.HandleInfo[]

    constructor(data: DataView, handleInfos: FX.HandleInfo[]) {
        super(data)
        this.handleInfos = handleInfos
    }

    closeHandles(): void {
        for (let i = 0; i < this.handleInfos.length; ++i) {
            this.handleInfos[i].handle.close()
        }
    }

    toString() {
        return `IncomingMessage(numBytes=${this.data.byteLength}, numHandles=${this.handleInfos.length})`
    }

    // TODO: static fromReadEtcResult(result: ReadEtcResult) {}
}

export class OutgoingMessage extends BaseMessage {
    public handleDispositions: FX.HandleDisposition[]

    constructor(data: DataView, handleDispositions: FX.HandleDisposition[]) {
        super(data)
        this.handleDispositions = handleDispositions
    }

    set txid(value: number) {
        this.data.setUint32(kMessageTxidOffset, value, true)
    }

    closeHandles(): void {
        for (let i = 0; i < this.handleDispositions.length; ++i) {
            this.handleDispositions[i].handle.close()
        }
    }

    toString() {
        return `OutgoingMessage(numBytes=${this.data.byteLength}, numHandles=${this.handleDispositions.length})`
    }
}

/** Encodes a FIDL message that contains a single parameter. */
export function encodeMessage<T>(encoder: Encoder, inlineSize: number, ty: MemberType, value: T): void {
    encoder.alloc(inlineSize, 0)
    ty.encode(encoder, value, kMessageHeaderSize, 1)
}

/** Encodes a FIDL message with multiple parameters.  The callback parameter
 * provides a decoder that is initialized on the provided Message, which
 * callers can use to decode specific types.  This functionality (encoding
 * multiple parameters) is implemented using a callback because each call to
 * MemberType.encode() must pass in a concrete type, rather than an element
 * popped from a List<FidlType>.
 */
export function encodeMessageWithCallback(encoder: Encoder, inlineSize: number, f: () => void): void {
    encoder.alloc(inlineSize, 0)
    f()
}

function validateDecoding(decoder: Decoder): void {
    // The ordering of the following two checks is important: if there is both unclaimed memory and
    // unclaimed handles, we should do the unclaimed handles clean up first (namely, closing all open)
    // handles.
    if (decoder.countUnclaimedHandles() > 0) {
        // If there are unclaimed handles at the end of the decoding, close all
        // handles to the best of our ability, and throw an error.
        for (const handleInfo in decoder.handleInfos) {
            try {
                handleInfo.handle.close()
                // ignore: avoid_catches_without_on_clauses
            } catch (e) {
                // best effort
            }
        }
        const unclaimed = decoder.countUnclaimedHandles()
        const total = decoder.handleInfos.length

        throw new FidlError(
            `Message contains extra handles (unclaimed: ${unclaimed}, total: ${total})`,
            FidlErrorCode.fidlTooManyHandles
        )
    }

    if (decoder.countUnclaimedBytes() > 0) {
        let unclaimed = decoder.countUnclaimedBytes()
        let total = decoder.data.byteLength
        throw new FidlError(
            `Message contains unread bytes (unclaimed: ${unclaimed}, total: ${total})`,
            FidlErrorCode.fidlTooManyBytes
        )
    }
}

/** Decodes a FIDL message that contains a single parameter. */
export function decodeMessage<T>(message: IncomingMessage, inlineSize: number, typ: MemberType): T {
    return decodeMessageWithCallback(message, inlineSize, (decoder: Decoder, offset: number) => {
        return typ.decode(decoder, offset, 1)
    })
}

/// Decodes a FIDL message with multiple parameters.  The callback parameter
/// provides a decoder that is initialized on the provided Message, which
/// callers can use to decode specific types.  The return result of the callback
/// (e.g. the decoded parameters, wrapped in a containing class/struct) is
/// returned as the result of the function.  This functionality (decoding
/// multiple parameters) is implemented using a callback because returning a
/// list would be insufficient: the list would be of type List<FidlType>,
/// whereas we want to retain concrete types of each decoded parameter.  The
/// only way to accomplish this in Dart is to pass in a function that collects
/// these multiple values into a bespoke, properly typed class.
export function decodeMessageWithCallback<T>(message: IncomingMessage, inlineSize: number, f: DecodeMessageCallback<T>): T {
    const size = kMessageHeaderSize + inlineSize
    const decoder = Decoder.fromMessage(message)
    decoder.claimBytes(size, 0)
    const out = f(decoder, kMessageHeaderSize)
    const padding = align(size) - size
    decoder.checkPadding(size, padding)
    validateDecoding(decoder)
    return out
}

export type DecodeMessageCallback<T> = (decoder: Decoder, offset: number) => T
export type IncomingMessageSink = (message: IncomingMessage) => void
export type OutgoingMessageSink = (message: OutgoingMessage) => void

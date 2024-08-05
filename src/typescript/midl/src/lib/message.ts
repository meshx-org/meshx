// Copyright 2023 The MeshX Authors. All rights reserved.
// Copyright 2022 The Fuchsia Authors. All rights reserved.
// Use of this source code is governed by a BSD-style license that can be
// found in the LICENSE file.

import { HandleDisposition, HandleInfo, ReadEtcResult } from "@meshx-org/fiber-ts"
import {
    FX_RIGHT_GET_PROPERTY,
    FX_RIGHT_INSPECT,
    FX_RIGHT_READ,
    FX_RIGHT_TRANSFER,
    FX_RIGHT_WAIT,
    Status,
} from "@meshx-org/fiber-types"
import { align, Decoder, Encoder } from "."
import { MidlError, ErrorCode } from "./errors"

// TODO: implement it
class MemberType {
    public decode(decoder: any, offset: any, padding: number): any {
        throw "not implemented"
    }

    public encode(encoder: any, value: any, offset: any, padding: number) {
        throw "not implemented"
    }
}

export const kMessageHeaderSize = 16
export const kLargeMessageInfoSize = 16
export const kLargeMessageVmoRights =
    FX_RIGHT_GET_PROPERTY | FX_RIGHT_INSPECT | FX_RIGHT_READ | FX_RIGHT_TRANSFER | FX_RIGHT_WAIT

export const kMessageTxidOffset = 0
export const kMessageFlagOffset = 4
export const kMessageDynamicFlagOffset = 6
export const kMessageMagicOffset = 7
export const kMessageOrdinalOffset = 8
export const kMagicNumberInitial = 1
export const kWireFormatV2FlagMask = 2
export const DYNAMIC_FLAGS_FLEXIBLE = 0x80
export const DYNAMIX_FLAGS_BYTEOVERFLOW = 0x40

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
            return DYNAMIC_FLAGS_FLEXIBLE
    }
}

/** Extract the CallStrictness from the dynamic flags byte of the message header. */
export function strictnessFromFlags(dynamicFlags: number): CallStrictness {
    if ((dynamicFlags & DYNAMIC_FLAGS_FLEXIBLE) != 0) {
        return CallStrictness.flexible
    }
    return CallStrictness.strict
}

/** Extract the CallOverflowing from the dynamic flags byte of the message header. */
export function overflowingFromFlags(dynamicFlags: number): CallOverflowing {
    if ((dynamicFlags & DYNAMIX_FLAGS_BYTEOVERFLOW) != 0) {
        return CallOverflowing.large
    }
    return CallOverflowing.small
}

class BaseMessage {
    public buffer: Uint8Array
    public data: DataView

    constructor(bytes: Uint8Array) {
        this.buffer = bytes
        this.data = new DataView(bytes.buffer)
    }

    get txid() {
        return this.data.getUint32(kMessageTxidOffset, true)
    }

    get ordinal() {
        return this.data.getBigUint64(kMessageOrdinalOffset, true)
    }

    get magic() {
        return this.data.getUint8(kMessageMagicOffset)
    }

    //parseWireFormat(): WireFormat {
    //    if ((this.data.getUint8(kMessageFlagOffset) & kWireFormatV2FlagMask) != 0) {
    //        return WireFormat.v2
    //    }

    //    throw new MidlError("unknown wire format", ErrorCode.UnsupportedWireFormat)
    //}

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
                    printable.push(".")
                }
            }

            buffer.push(`${hex.toString().padEnd(3 * width)} ${printable}\n`)
        }

        console.log(
            "==================================================\n" +
                buffer +
                "=================================================="
        )
    }
}

export class IncomingMessage extends BaseMessage {
    public handleInfos: HandleInfo[]

    constructor(bytes: Uint8Array, handleInfos: HandleInfo[]) {
        super(bytes)
        this.handleInfos = handleInfos
    }

    static fromReadEtcResult(result: ReadEtcResult): IncomingMessage {
        if (result.status !== Status.OK) {
            throw new Error(`ony results with Status.OK can be used to create a message`)
        }

        return new IncomingMessage(result.bytes!, result.handleInfos!)
    }

    closeHandles(): void {
        for (let i = 0; i < this.handleInfos.length; ++i) {
            this.handleInfos[i].handle.close()
        }
    }

    override toString() {
        return `IncomingMessage(numBytes=${this.data.byteLength}, numHandles=${this.handleInfos.length})`
    }

    // TODO: static fromReadEtcResult(result: ReadEtcResult) {}
}

export class OutgoingMessage extends BaseMessage {
    public handleDispositions: HandleDisposition[]

    constructor(bytes: Uint8Array, handleDispositions: HandleDisposition[]) {
        super(bytes)
        this.handleDispositions = handleDispositions
    }

    override set txid(value: number) {
        this.data.setUint32(kMessageTxidOffset, value, true)
    }

    closeHandles(): void {
        for (let i = 0; i < this.handleDispositions.length; ++i) {
            this.handleDispositions[i].handle?.close()
        }
    }
}

const DYNAMIX_FLAGS_FLEXIBLE = 0x80

 

 

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
        for (const handleInfo of decoder.handleInfos) {
            try {
                handleInfo.handle.close()
            } catch (e) {
                // best effort
            }
        }
        const unclaimed = decoder.countUnclaimedHandles()
        const total = decoder.handleInfos.length

        throw new MidlError(
            `Message contains extra handles (unclaimed: ${unclaimed}, total: ${total})`,
            ErrorCode.TooManyHandles
        )
    }

    if (decoder.countUnclaimedBytes() > 0) {
        const unclaimed = decoder.countUnclaimedBytes()
        const total = decoder.data.byteLength
        throw new MidlError(
            `Message contains unread bytes (unclaimed: ${unclaimed}, total: ${total})`,
            ErrorCode.TooManyBytes
        )
    }
}

/** Decodes a FIDL message that contains a single parameter. */
export function decodeMessage<T>(message: IncomingMessage, inlineSize: number, typ: MemberType): T {
    return decodeMessageWithCallback(message, inlineSize, (decoder: Decoder, offset: number) => {
        return typ.decode(decoder, offset, 1)
    })
}

/// Decodes a MIDL message with multiple parameters.  The callback parameter
/// provides a decoder that is initialized on the provided Message, which
/// callers can use to decode specific types.  The return result of the callback
/// (e.g. the decoded parameters, wrapped in a containing class/struct) is
/// returned as the result of the function.  This functionality (decoding
/// multiple parameters) is implemented using a callback because returning a
/// list would be insufficient: the list would be of type List<FidlType>,
/// whereas we want to retain concrete types of each decoded parameter.  The
/// only way to accomplish this in Dart is to pass in a function that collects
/// these multiple values into a bespoke, properly typed class.
export function decodeMessageWithCallback<T>(
    message: IncomingMessage,
    inlineSize: number,
    f: DecodeMessageCallback<T>
): T {
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

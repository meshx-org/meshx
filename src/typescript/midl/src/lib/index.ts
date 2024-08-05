/* eslint-disable @typescript-eslint/no-inferrable-types */
/* eslint-disable @typescript-eslint/no-non-null-assertion */
import * as fiber from "@meshx-org/fiber-ts"
import * as midl from "./midl"
import { ErrorCode, MidlError, MidlIntOutOfRangeError } from "./errors"
import {
    ChannelType,
    HandleType,
    MidlType,
    SimpleMidlType,
    UnknownRawDataType,
    maybeThrowOnUnknownHandles,
} from "./types"
import { UnknownRawData } from "./unknown"
import { UnionFactory } from "./union"
import { CallStrictness, IncomingMessage, OutgoingMessage, kMagicNumberInitial, kMessageDynamicFlagOffset, kMessageFlagOffset, kMessageHeaderSize, kMessageMagicOffset, kMessageOrdinalOffset, kMessageTxidOffset, kWireFormatV2FlagMask, strictnessToFlags } from "./message"

const ALIGMENT = 8
const ALIGMENT_MASK = ALIGMENT - 1

const ENVELOPE_INLINE_MARKER = 1
const ENVELOPE_OUT_OF_LINE_MARKER = 0
const ENVELOPE_INLINE_CONTENT_SIZE = 4
/// The maximum recursion depth of encoding and decoding.
/// Each nested aggregate type (structs, unions, arrays, or vectors) counts as one step in the
/// recursion depth.
const MAX_RECURSION = 32
const MAX_OUT_OF_LINE_DEPTH = 32

/// Indicates that an optional value is present.
const ALLOC_PRESENT_U64 = 18_446_744_073_709_551_615n
/// Indicates that an optional value is present.
const ALLOC_PRESENT_U32 = 4_294_967_295
/// Indicates that an optional value is absent.
const ALLOC_ABSENT_U64: bigint = 0n

const MIN_BUFFER_SIZE_INCREASE_FACTOR = 2

const textEncoder = new TextEncoder()

export function align(size: number) {
    return (size + ALIGMENT_MASK) & ~ALIGMENT_MASK
}

function _checkIntRange(value: number, min: number, max: number): void {
    if (value < min || value > max) {
        throw new MidlIntOutOfRangeError(value, min, max)
    }
}

/**
 * Rounds `x` up if necessary so that it is a multiple of `align`.
 * Requires `align` to be a (nonzero) power of two.
 */
function roundUpToAlign(x: number, align: number): number {
    if (align === 0) {
        throw new Error("Align must be nonzero")
    }

    if ((align & (align - 1)) !== 0) {
        throw new Error("Align must be a power of two")
    }

    return (x + align - 1) & ~(align - 1)
}

export abstract class Union {
    get $ordinal(): number {
        throw new Error("must be implemented")
    }

    get $data(): unknown {
        throw new Error("must be implemented")
    }
}

enum EnvelopePresence {
    present,
    absent,
}

enum EnvelopeContentLocation {
    inline,
    outOfLine,
}

type EnvelopeHeader = {
    numBytes: number
    numHandles: number
    presence: EnvelopePresence
    contentLocation: EnvelopeContentLocation
}

export abstract class Enum {
    $value: number = 0
}

export abstract class Struct {
    $encode($encoder: Encoder, $offset: number, $depth: number) {
        throw new Error("must be implemented")
    }

    static $decode($decoder: Decoder, $offset: number) {
        throw new Error("must be implemented")
    }
}

export class Encoder {
    public buffer: Uint8Array
    public data: DataView

    #handleDispositions
    #extent

    private constructor(bytes: Uint8Array, handles: any[]) {
        this.buffer = bytes
        this.data = new DataView(bytes.buffer)
        this.#handleDispositions = handles
        this.#extent = 0
    }

    private grow(newSize: number): void {
        const newList = new Uint8Array(newSize)
        newList.set(new Uint8Array(this.data.buffer))
        this.data = new DataView(newList.buffer)
        this.buffer = newList
    }

    private claimBytes(claimSize: number): void {
        this.#extent += claimSize
        if (this.#extent > this.data.byteLength) {
            const newSize = Math.max(this.#extent, MIN_BUFFER_SIZE_INCREASE_FACTOR * this.data.byteLength)
            this.grow(newSize)
        }
    }

    public alloc(size: number, nextOutOfLineDepth: number): number {
        if (nextOutOfLineDepth > MAX_OUT_OF_LINE_DEPTH) {
            throw new MidlError("Exceeded maxOutOfLineDepth", ErrorCode.ExceededMaxOutOfLineDepth)
        }

        const offset = this.#extent
        this.claimBytes(align(size))
        return offset
    }

    debugCheckBounds(offset: number, size: number) {
        if (offset + size > this.data.byteLength) {
            throw new Error("Buffer overflow")
        }
    }

    get message() {
        const trimmed = this.buffer.subarray(0, this.#extent)
        return new OutgoingMessage(trimmed, this.#handleDispositions)
    }

    static encode<V>(handles: any[], x: V, type: midl.Encodable<V>): Uint8Array {
        const encoder = new Encoder(new Uint8Array(type.inlineSize), handles)
        type.encode(encoder, x, 0, 0)
        return encoder.buffer
    }

    encodeMessageHeader(ordinal: bigint, txid: number, strictness: CallStrictness): void {
        this.alloc(kMessageHeaderSize, 0)
        this.encodeUInt32(txid, kMessageTxidOffset)
        this.encodeUInt8(kWireFormatV2FlagMask, kMessageFlagOffset)
        this.encodeUInt8(0, kMessageFlagOffset + 1)
        this.encodeUInt8(strictnessToFlags(strictness), kMessageDynamicFlagOffset)
        this.encodeUInt8(kMagicNumberInitial, kMessageMagicOffset)
        this.encodeUInt64(ordinal, kMessageOrdinalOffset)
    }

    addHandleDisposition(value: HandleDisposition) {
        this.#handleDispositions.push(value)
    }

    encodeString(value: string, offset: number, depth: number) {
        const utf8 = textEncoder.encode(value)
        encodeVectorFromBytes(this, offset, depth, utf8)
    }

    encodeUnion<T extends Union>(
        value: T,
        offset: number,
        depth: number,
        members: Record<number, MidlType<unknown, unknown[]>>,
        flexible: boolean,
        resource: boolean
    ) {
        //assert(this.wireFormat == WireFormat.v2);
        const envelopeOffset = offset + 8
        const ordinal = value.$ordinal

        if (ordinal > Number.MAX_SAFE_INTEGER) {
            throw new MidlError(`Invalid ordinal: ${ordinal}`, ErrorCode.ExceededSafeIntegerLimit)
        }

        if (ordinal === 0) {
            throw new MidlError(`Invalid ordinal: ${ordinal}`, ErrorCode.Unknown)
        }

        let fieldType = members[Number(ordinal)] // UNSAFE cast to number
        const data = value.$data

        if (fieldType === null && flexible && data instanceof UnknownRawData) {
            maybeThrowOnUnknownHandles(resource, data)
            fieldType = new UnknownRawDataType(data.data.length, data.handles.length)
        }

        if (fieldType === null) {
            throw new MidlError(`Bad xunion ordinal: ${ordinal}`, ErrorCode.StrictUnionUnknownField)
        }

        this.encodeUInt64(BigInt(ordinal), offset)
        this.encodeEnvelopePresent(envelopeOffset, depth, data, fieldType)
    }

    private nextOffset() {
        return this.#extent
    }

    private countHandles() {
        return this.#handleDispositions.length
    }

    private encodeEnvelopePresent<T>(offset: number, depth: number, field: T, fieldType: MidlType<T, T[]>) {
        const fieldSize = fieldType.inlineSize

        if (fieldSize <= 4) {
            const initialNumHandles = this.countHandles()
            fieldType.encode(this, field, offset, depth)
            const numHandles = this.countHandles() - initialNumHandles

            this.encodeUInt16(numHandles, offset + 4)
            this.encodeUInt16(ENVELOPE_INLINE_MARKER, offset + 6)
        } else {
            const initialNumHandles = this.countHandles()
            const fieldOffset = this.alloc(fieldSize, depth)
            fieldType.encode(this, field, fieldOffset, depth + 1)
            const numHandles = this.countHandles() - initialNumHandles
            const numBytes = this.nextOffset() - fieldOffset
            this.encodeUInt32(numBytes, offset)
            this.encodeUInt16(numHandles, offset + 4)
            this.encodeUInt16(ENVELOPE_OUT_OF_LINE_MARKER, offset + 6)
        }
    }

    encodeBool(value: boolean, offset: number): void {
        this.data.setInt8(offset, value ? 1 : 0)
    }

    encodeInt8(value: number, offset: number): void {
        _checkIntRange(value, -128, 127)
        this.data.setInt8(offset, value)
    }

    encodeUInt8(value: number, offset: number): void {
        _checkIntRange(value, 0, 255)
        this.data.setUint8(offset, value)
    }

    encodeInt16(value: number, offset: number): void {
        _checkIntRange(value, -32768, 32767)
        this.data.setInt16(offset, value, true)
    }

    encodeUInt16(value: number, offset: number): void {
        _checkIntRange(value, 0, 65535)
        this.data.setUint16(offset, value, true)
    }

    encodeInt32(value: number, offset: number): void {
        _checkIntRange(value, -2147483648, 2147483647)
        this.data.setInt32(offset, value, true)
    }

    encodeUInt32(value: number, offset: number): void {
        _checkIntRange(value, 0, 4294967295)
        this.data.setUint32(offset, value, true)
    }

    encodeInt64(value: bigint, offset: number): void {
        this.data.setBigInt64(offset, value, true)
    }

    encodeUInt64(value: bigint, offset: number): void {
        this.data.setBigUint64(offset, value, true)
    }

    encodeFloat32(value: number, offset: number): void {
        this.data.setFloat32(offset, value, true)
    }

    encodeFloat64(value: number, offset: number): void {
        this.data.setFloat64(offset, value, true)
    }

    checkRecursionDepth(depth: number) {
        if (depth > 32) {
            throw new Error("Maximum recursion depth exceeded")
        }
    }

    /** Append bytes to the very end (out-of-line) of the buffer.*/
    appendOutOfLineBytes(bytes: Uint8Array) {
        if (bytes.length == 0) {
            return
        }

        const start = this.buffer.length
        const end = this.buffer.length + roundUpToAlign(bytes.length, 8)

        // Safety:
        // - this.buffer is initially uninitialized when resized, but it is then
        //   initialized by a later copy so it leaves this block initialized.
        // - There is enough room for the 8 byte padding filler because end's
        //   alignment is rounded up to 8 bytes and bytes.length != 0.
        this.grow(end)

        // Zero the last 8 bytes in the new buffer
        this.buffer.fill(0, end - 8, end)

        // Copy the new bytes to the end of the buffer
        this.buffer.set(bytes, start)

        //this.data.setBigUint64(end - 8, 0n, true)

        //console.log(this.buffer.subarray(start), bytes)
        //let padding_ptr = self.buf.get_unchecked_mut(end - 8);
        //mem::transmute::<*mut u8, *mut u64>(padding_ptr).write_unaligned(0);

        //copyNonOverlapping(bytes, this.buffer.subarray(start), start, bytes.length)
    }
}

function encodeVectorFromBytes(
    encoder: Encoder,
    offset: number,
    recursionDepth: number,
    slice?: Uint8Array | null
): void {
    if (slice) {
        // Two u64: (len, present)
        encoder.encodeUInt64(BigInt(slice.length), offset)
        encoder.encodeUInt64(ALLOC_PRESENT_U64, offset + 8)
        encoder.checkRecursionDepth(recursionDepth + 1)
        encoder.appendOutOfLineBytes(slice)
    } else {
        encodeAbsentVector(encoder, offset, recursionDepth)
    }
}

/// Encode an missing vector-like component.
function encodeAbsentVector(encoder: Encoder, offset: number, recursionDepth: number) {
    encoder.encodeUInt64(0n, offset)
    encoder.encodeUInt64(ALLOC_ABSENT_U64, offset + 8)
}

function decodeVectorHeader(decoder: Decoder, offset: number): number | null {
    try {
        const len = decoder.decodeUInt64(offset)
        const present = decoder.decodeUInt64(offset + 8)

        if (present === ALLOC_PRESENT_U64) {
            const lenNumber = Number(len)
            if (lenNumber <= Number.MAX_SAFE_INTEGER && lenNumber <= decoder.buffer.length) {
                return lenNumber
            } else {
                throw "Error.OutOfRange"
                //return Error.OutOfRange
            }
        } else if (present === ALLOC_ABSENT_U64) {
            if (len === 0n) {
                return null
            } else {
                throw "Error.UnexpectedNullRef"
                //return Error.UnexpectedNullRef
            }
        } else {
            throw "Error.InvalidPresenceIndicator"
            //return Error.InvalidPresenceIndicator
        }
    } catch {
        throw "Error.InvalidPresenceIndicator"
        //return Error.InvalidPresenceIndicator
    }
}

type HandleInfo = any
type HandleDisposition = any

export class Decoder {
    /// Buffer from which to read data.
    public buffer: Uint8Array
    public data: DataView

    /// Buffer from which to read handles.
    public handleInfos!: HandleInfo[]

    /// Next out of line block in buf.
    private nextOutOfLine: number

    #depth: number = 0
    #nextOffset = 0
    #nextHandle = 0

    private constructor(bytes: Uint8Array, nextOutOfLine: number) {
        this.buffer = bytes
        this.nextOutOfLine = nextOutOfLine
        this.data = new DataView(bytes.buffer)
    }

    static fromMessage(message: IncomingMessage): Decoder {
        const decoder = new Decoder(message.buffer, 0)
        return decoder
    }

    nextOffset() {
        return this.#nextOffset
    }

    countClaimedHandles(): number {
        return this.#nextHandle
    }

    countUnclaimedHandles(): number {
        return this.handleInfos.length - this.#nextHandle
    }

    countUnclaimedBytes(): number {
        return this.data.byteLength - this.#nextOffset
    }

    public claimBytes(size: number, nextOutOfLineDepth: number): number {
        if (nextOutOfLineDepth > MAX_OUT_OF_LINE_DEPTH) {
            throw new MidlError("Exceeded maxOutOfLineDepth", ErrorCode.ExceededMaxOutOfLineDepth)
        }

        const result = this.#nextOffset
        this.#nextOffset += align(size)

        if (this.#nextOffset > this.data.byteLength) {
            throw new MidlError("Cannot access out of range memory", ErrorCode.TooFewBytes)
        }

        return result
    }

    claimHandle(): HandleInfo {
        if (this.#nextHandle >= this.handleInfos.length) {
            throw new MidlError("Cannot access out of range handle", ErrorCode.TooFewHandles)
        }
        return this.handleInfos[this.#nextHandle++]
    }

    checkPadding(offset: number, padding: number) {
        for (let readAt = offset; readAt < offset + padding; readAt++) {
            if (this.data.getUint8(readAt) != 0) {
                throw new MidlError(`Non-zero padding at: ${readAt}`, ErrorCode.InvalidPaddingByte)
            }
        }
    }

    decodeString(offset: number): string {
        //this.debugCheckBounds::<Self>(offset);
        const len = decodeVectorHeader(this, offset)
        if (len === null) {
            return ""
        }

        let decodedString = ""

        this.readOutOfLine(len, (decoder, offset) => {
            const bytes = decoder.buffer.subarray(offset, offset + len)
            console.log(bytes)
            const utf8 = new TextDecoder("utf-8").decode(bytes)
            decodedString = utf8

            return true
        })

        return decodedString
    }

    private decodeEnvelopeContent<T>(
        header: EnvelopeHeader,
        headerOffset: number,
        fieldType: MidlType<T, T[]> | null,
        depth: number,
        isEmpty: boolean
    ): T | null {
        switch (header.presence) {
            case EnvelopePresence.present: {
                if (isEmpty) throw new MidlError("expected empty envelope")

                if (header.contentLocation == EnvelopeContentLocation.inline) {
                    if (fieldType !== null) {
                        if (fieldType.inlineSize > 4) {
                            throw new MidlError(
                                "received inline data, but field size indicates an out-of-line payload",
                                ErrorCode.InvalidInlineBitInEnvelope
                            )
                        }

                        const claimedHandles = this.countClaimedHandles()
                        const fieldInlineSize = fieldType.inlineSize
                        const field = fieldType.decode(this, headerOffset, depth + 1)
                        this.checkPadding(
                            headerOffset + fieldInlineSize,
                            ENVELOPE_INLINE_CONTENT_SIZE - fieldInlineSize
                        )
                        const numHandlesConsumed = this.countClaimedHandles() - claimedHandles
                        if (header.numHandles != numHandlesConsumed) {
                            throw new MidlError(
                                "envelope handles were mis-sized",
                                ErrorCode.InvalidNumHandlesInEnvelope
                            )
                        }

                        return field
                    }
                    for (let i = 0; i < header.numHandles; i++) {
                        const handleInfo = this.claimHandle()
                        try {
                            handleInfo.handle.close()
                            // ignore: avoid_catches_without_on_clauses
                        } catch (e) {
                            // best effort
                        }
                    }
                    return null
                }

                if (fieldType !== null) {
                    if (fieldType.inlineSize <= 4) {
                        throw new MidlError(
                            "received out-of-line data, but field size indicates an inline payload",
                            ErrorCode.InvalidInlineBitInEnvelope
                        )
                    }

                    const fieldInlineSize = fieldType.inlineSize
                    const fieldOffset = this.claimBytes(fieldInlineSize, depth)
                    const claimedHandles = this.countClaimedHandles()
                    const field = fieldType.decodeObject(this, fieldOffset, fieldInlineSize, depth + 1)
                    const numBytesConsumed = this.nextOffset() - fieldOffset
                    const numHandlesConsumed = this.countClaimedHandles() - claimedHandles

                    if (header.numHandles !== numHandlesConsumed) {
                        throw new MidlError("envelope handles were mis-sized", ErrorCode.InvalidNumHandlesInEnvelope)
                    }

                    if (header.numBytes !== numBytesConsumed) {
                        throw new MidlError("envelope was mis-sized", ErrorCode.InvalidNumBytesInEnvelope)
                    }

                    return field
                }

                this.claimBytes(header.numBytes, depth)
                for (let i = 0; i < header.numHandles; i++) {
                    const handleInfo = this.claimHandle()
                    try {
                        handleInfo.handle.close()
                        // ignore: avoid_catches_without_on_clauses
                    } catch (e) {
                        // best effort
                    }
                }
                return null
            }
            case EnvelopePresence.absent: {
                if (header.numBytes !== 0) {
                    throw new MidlError("absent envelope with non-zero bytes", ErrorCode.InvalidNumBytesInEnvelope)
                }

                if (header.numHandles !== 0) {
                    throw new MidlError("absent envelope with non-zero handles", ErrorCode.InvalidNumHandlesInEnvelope)
                }

                return null
            }
        }

        return null
    }

    private decodeEnvelopeHeader(offset: number): EnvelopeHeader {
        const numHandles = this.decodeUInt16(offset + 4)

        switch (this.decodeUInt16(offset + 6)) {
            case 0: {
                // out of line content
                const numBytes = this.decodeUInt32(offset)

                if (numBytes % 8 != 0) {
                    throw new MidlError("improperly aligned byte count", ErrorCode.InvalidNumBytesInEnvelope)
                }

                return {
                    numBytes,
                    numHandles,
                    presence: numBytes != 0 || numHandles != 0 ? EnvelopePresence.present : EnvelopePresence.absent,
                    contentLocation: EnvelopeContentLocation.outOfLine,
                }
            }
            case 1: // inlined content
                return {
                    numBytes: ENVELOPE_INLINE_CONTENT_SIZE,
                    numHandles,
                    presence: EnvelopePresence.present,
                    contentLocation: EnvelopeContentLocation.inline,
                }
            default:
                throw new MidlError("invalid inline marker in envelope", ErrorCode.InvalidInlineMarkerInEnvelope)
        }
    }

    private decodeEnvelope<T>(
        offset: number,
        depth: number,
        fieldType: SimpleMidlType<T> | null,
        isEmpty = false
    ): T | null {
        const header = this.decodeEnvelopeHeader(offset)
        return this.decodeEnvelopeContent(header, offset, fieldType, depth, isEmpty)
    }

    decodeUnion<T>(
        offset: number,
        depth: number,
        members: Record<number, MidlType<unknown, unknown[]>>,
        ctor: UnionFactory<T>,
        flexible: boolean,
        resource: boolean
    ): T | null {
        //assert(this.wireFormat == WireFormat.v2)
        const envelopeOffset = offset + 8
        const ordinal = Number(this.decodeUInt64(offset))

        if (ordinal == 0) {
            this.decodeEnvelope(envelopeOffset, depth, null, true)
            return null
        } else {
            const header = this.decodeEnvelopeHeader(envelopeOffset)
            const fieldType = members[ordinal]

            if (fieldType == null) {
                const unknownData = this.decodeEnvelopeContent(
                    header,
                    envelopeOffset,
                    new UnknownRawDataType(header.numBytes, header.numHandles),
                    depth,
                    false
                )
                if (unknownData == null) throw new MidlError("Bad xunion: missing content")

                if (!flexible) {
                    unknownData.closeHandles()
                    throw new MidlError(`Bad xunion ordinal: ${ordinal}`, ErrorCode.StrictUnionUnknownField)
                }

                maybeThrowOnUnknownHandles(resource, unknownData)
                return ctor(ordinal, unknownData)
            }

            const field = this.decodeEnvelopeContent(header, envelopeOffset, fieldType, depth, false)
            if (field == null) {
                throw new MidlError("Bad xunion: missing content")
            }

            return ctor(ordinal, field)
        }
    }

    decodeBool(offset: number): boolean {
        switch (this.data.getUint8(offset)) {
            case 0:
                return false
            case 1:
                return true
            default:
                throw new MidlError("Invalid boolean", ErrorCode.InvalidBoolean)
        }
    }

    decodeInt8 = (offset: number) => this.data.getInt8(offset)
    decodeUInt8 = (offset: number) => this.data.getUint8(offset)
    decodeInt16 = (offset: number) => this.data.getInt16(offset, true)
    decodeUInt16 = (offset: number) => this.data.getUint16(offset, true)
    decodeInt32 = (offset: number) => this.data.getInt32(offset, true)
    decodeUInt32 = (offset: number) => this.data.getUint32(offset, true)
    decodeInt64 = (offset: number) => this.data.getBigInt64(offset, true)
    decodeUInt64 = (offset: number) => this.data.getBigUint64(offset, true)
    decodeFloat32 = (offset: number) => this.data.getFloat32(offset, true)
    decodeFloat64 = (offset: number) => this.data.getFloat64(offset, true)

    /// Runs the provided closure inside an decoder modified to read out-of-line data.
    readOutOfLine<R>(len: number, f: (dec: Decoder, offset: number) => R) {
        const offset = this.nextOutOfLine
        const alignedLen = roundUpToAlign(len, 8)
        this.nextOutOfLine = this.nextOutOfLine + alignedLen

        if (this.nextOutOfLine > this.buffer.length) {
            throw new Error("OutOfRange")
        }

        if (this.nextOutOfLine < 8) {
            throw new Error("Invalid nextOutOfLine value")
        }

        const lastU64Ptr = this.data.getBigUint64(this.nextOutOfLine - 8, true)

        const padding = alignedLen - len
        const mask = ~((0n << BigInt(padding * 8)) - 1n)

        if (lastU64Ptr & BigInt(mask !== 0n)) {
            console.log(lastU64Ptr, BigInt(mask !== 0n), "m:", mask, "p:", padding)
            throw this.endOfBlockPaddingError(offset + len, this.nextOutOfLine)
        }

        this.#depth += 1
        if (this.#depth > MAX_RECURSION) {
            throw new Error("MaxRecursionDepth")
        }

        try {
            const res = f(this, offset)
            this.#depth -= 1
            return res
        } catch (e) {
            this.#depth -= 1
            throw e
        }
    }

    /// Generates an error for bad padding bytes at the end of a block.
    /// Assumes that it is already known that there is an error.
    endOfBlockPaddingError(start: number, end: number): Error {
        for (let i = start; i < end; i++) {
            if (this.buffer[i] !== 0) {
                return new Error(`NonZeroPadding: padding_start=${start}`)
            }
        }

        throw new Error("Invalid padding bytes detected, but missing when generating error")
    }

    static decode<V>(buffer: Uint8Array, handles: any[], type: MidlType<V, V[]>): V {
        const nextOutOfLine = roundUpToAlign(type.inlineSize, 8)
        if (nextOutOfLine > buffer.length) {
            throw new Error("Error::OutOfRange")
        }

        const decoder = new Decoder(buffer, nextOutOfLine)
        return type.decode(decoder, 0, 0)
    }
}

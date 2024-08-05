import { Decoder, Encoder, Enum, Struct, Union, align } from "."
import { MidlError, ErrorCode } from "./errors"
import { Decodable, Encodable } from "./midl"
import {
    FX_DEFAULT_CHANNEL_RIGHTS,
    FX_HANDLE_OP_MOVE,
    FX_OBJ_TYPE_CHANNEL,
    FX_OBJ_TYPE_NONE,
    FX_RIGHT_SAME_RIGHTS,
    fx_obj_type_t,
} from "@meshx-org/fiber-types"
import * as fiber from "@meshx-org/fiber-ts"
import { UnknownRawData } from "./unknown"
import { UnionFactory } from "./union"
import { InterfaceHandle, InterfaceRequest } from "./interface"

const HANDLE_ABSENT = 0
const HANDLE_PRESENT = 0xffffffff

export abstract class MidlType<T, I extends Iterable<T>> implements Encodable<T>, Decodable<T> {
    constructor(public readonly inlineSize: number) {}

    abstract decode(decoder: Decoder, offset: number, depth: number): T
    abstract encode(encoder: Encoder, value: T, offset: number, depth: number): void

    decodeObject(decoder: Decoder, offset: number, inlineSize: number, depth: number): T {
        const decoded = this.decode(decoder, offset, depth)
        const padding = align(inlineSize) - inlineSize
        decoder.checkPadding(offset + inlineSize, padding)
        return decoded
    }

    abstract decodeArray(decoder: Decoder, count: number, offset: number, depth: number): I
}

export abstract class SimpleMidlType<T> extends MidlType<T, Array<T>> {
    constructor(inlineSize: number) {
        super(inlineSize)
    }

    override decodeArray(decoder: Decoder, count: number, offset: number, depth: number): Array<T> {
        return []
        // List<T>.unmodifiable(Iterable<T>.generate(
        //   count, (int i) => decode(decoder, offset + i * inlineSize, depth)));
    }
}

export function maybeThrowOnUnknownHandles(resource: boolean, data: UnknownRawData) {
    if (!resource && data.handles.length !== 0) {
        data.closeHandles()
        throw new MidlError("Unknown data contained handles on encode", ErrorCode.NonResourceHandle)
    }
}

export class EnumType<V extends Enum> extends SimpleMidlType<V> {
    #underlying: SimpleMidlType<unknown>

    constructor(underlying: SimpleMidlType<unknown>) {
        super(underlying.inlineSize)
        this.#underlying = underlying
    }

    /** @internal */
    encode(encoder: Encoder, value: V, offset: number, depth: number): void {
        this.#underlying.encode(encoder, value.$value, offset, depth)
    }

    decode(decoder: Decoder, offset: number): V {
        throw new Error("Method not implemented.")
    }
}

export class StructType<V extends Struct> extends SimpleMidlType<V> {
    readonly #decode

    constructor(inlineSize: number, decode: (decoder: Decoder, offset: number, depth: number) => V) {
        super(inlineSize)
        this.#decode = decode
    }

    encode(encoder: Encoder, value: V, offset: number, depth: number): void {
        value.$encode(encoder, offset, depth)
    }

    decode(decoder: Decoder, offset: number, depth: number): V {
        return this.#decode(decoder, offset, depth)
    }
}

export class UnionType<V extends Union> extends SimpleMidlType<V> {
    constructor(
        private readonly members: Record<number, MidlType<any, any>>,
        private readonly ctor: UnionFactory<V>,
        private readonly flexible: boolean,
        private readonly resource: boolean
    ) {
        super(16)
    }

    encode(encoder: Encoder, value: V, offset: number, depth: number): void {
        encoder.encodeUnion(
            value,
            offset,
            depth,
            {
                1: new UInt32Type(),
            },
            true,
            false
        )
    }

    decode(decoder: Decoder, offset: number, depth: number): V {
        const value = decoder.decodeUnion(offset, depth, this.members, this.ctor, this.flexible, this.resource)
        if (value === null) {
            throw new MidlError("Found null for a non-nullable type", ErrorCode.NonNullableTypeWithNullValue)
        }
        return value
    }
}

export class UnknownRawDataType extends SimpleMidlType<UnknownRawData> {
    constructor(private numBytes: number, private numHandles: number) {
        super(numBytes + numBytes * 4)
    }

    encode(encoder: Encoder, value: any, offset: number, depth: number): void {
        throw new Error("Method not implemented.")
    }

    decode(decoder: Decoder, offset: number): UnknownRawData {
        throw new Error("Method not implemented.")
    }
}

export class UInt8Type extends SimpleMidlType<number> {
    constructor() {
        super(1)
    }

    encode(encoder: Encoder, value: number, offset: number, depth: number): void {
        encoder.encodeUInt8(value, offset)
    }

    decode(decoder: Decoder, offset: number, depth: number) {
        return decoder.decodeUInt8(offset)
    }
}

export class UInt32Type extends SimpleMidlType<number> {
    constructor() {
        super(4)
    }

    encode(encoder: Encoder, value: number, offset: number, depth: number) {
        encoder.encodeUInt32(value, offset)
    }

    decode(decoder: Decoder, offset: number) {
        return decoder.decodeUInt32(offset)
    }
}

export class UInt64Type extends SimpleMidlType<bigint> {
    constructor() {
        super(8)
    }

    encode(encoder: Encoder, value: bigint, offset: number, depth: number) {
        encoder.encodeUInt64(value, offset)
    }

    decode(decoder: Decoder, offset: number, depth: number) {
        return decoder.decodeUInt64(offset)
    }
}

export class BoolType extends SimpleMidlType<boolean> {
    constructor() {
        super(1)
    }

    encode(encoder: Encoder, value: boolean, offset: number, depth: number): void {
        encoder.encodeBool(value, offset)
    }

    decode(decoder: Decoder, offset: number, depth: number) {
        return decoder.decodeBool(offset)
    }
}

export class Int8Type extends SimpleMidlType<number> {
    constructor() {
        super(1)
    }

    encode(encoder: Encoder, value: number, offset: number, depth: number): void {
        encoder.encodeInt8(value, offset)
    }

    decode(decoder: Decoder, offset: number, depth: number) {
        return decoder.decodeInt8(offset)
    }
}

export class Int32Type extends SimpleMidlType<number> {
    constructor() {
        super(4)
    }

    encode(encoder: Encoder, value: number, offset: number, depth: number) {
        encoder.encodeInt32(value, offset)
    }

    decode(decoder: Decoder, offset: number) {
        return decoder.decodeInt32(offset)
    }
}

export class Int64Type extends SimpleMidlType<bigint> {
    constructor() {
        super(8)
    }

    encode(encoder: Encoder, value: bigint, offset: number, depth: number) {
        encoder.encodeInt64(value, offset)
    }

    decode(decoder: Decoder, offset: number, depth: number) {
        return decoder.decodeInt64(offset)
    }
}

export class StringType extends SimpleMidlType<string> {
    constructor() {
        super(16)
    }

    encode(encoder: Encoder, value: string, offset: number, depth: number): void {
        encoder.encodeString(value, offset, depth)
    }

    decode(decoder: Decoder, offset: number, depth: number) {
        return decoder.decodeString(offset)
    }
}

export class MethodType {
    constructor(
        public request: any | null,
        public response: any | null,
        public name: string,
        private reqInlineSize: number,
        private resInlineSize: number
    ) {}

    responseInlineSize() {
        return 0
    }

    requestInlineSize() {
        return 0
    }
}

export class MemberType<T> {
    constructor(type: SimpleMidlType<any>, offset: number) {
        this.type = type
        this.offset = offset
    }

    readonly type: SimpleMidlType<any>
    readonly offset: number

    encode(encoder: Encoder, value: T, base: number, depth: number): void {
        this.type.encode(encoder, value, base + this.offset, depth)
    }

    decode(decoder: Decoder, base: number, depth: number) {
        return this.type.decode(decoder, base + this.offset, depth)
    }
}

function encodeHandle(
    encoder: Encoder,
    value: fiber.HandleDisposition | null,
    offset: number,
    nullable: boolean
): void {
    const present = value !== null && value.handle.isValid

    if (!nullable && !present) {
        throw new MidlError("Found null for a non-nullable type", ErrorCode.NonNullableTypeWithNullValue)
    }

    encoder.encodeUInt32(present ? HANDLE_PRESENT : HANDLE_ABSENT, offset)

    if (present) {
        encoder.addHandleDisposition(value)
    }
}

function checkHandleRights(handleInfo: fiber.HandleInfo, objectType: number, rights: number): fiber.Handle {
    if (objectType !== FX_OBJ_TYPE_NONE && handleInfo.type !== FX_OBJ_TYPE_NONE && handleInfo.type !== objectType) {
        handleInfo.handle.close()
        throw new MidlError(
            `Handle has object type ${handleInfo.type} but required $objectType.`,
            ErrorCode.IncorrectHandleType
        )
    }

    if (rights != FX_RIGHT_SAME_RIGHTS && handleInfo.rights != FX_RIGHT_SAME_RIGHTS) {
        if ((rights & ~handleInfo.rights) != 0) {
            handleInfo.handle.close()
            throw new MidlError(
                `Required handle rights were missing. Got ${handleInfo.rights}, want $rights.`,
                ErrorCode.MissingRequiredHandleRights
            )
        }

        if ((handleInfo.rights & ~rights) != 0) {
            return handleInfo.handle.replace(rights)
        }
    }

    return handleInfo.handle
}

function decodeNullableHandle(
    decoder: Decoder,
    offset: number,
    objectType: number,
    rights: number
): fiber.Handle | null {
    const encoded = decoder.decodeUInt32(offset)

    if (encoded != HANDLE_ABSENT && encoded !== HANDLE_PRESENT) {
        throw new MidlError(`Invalid handle encoding: ${encoded}.`)
    }

    if (encoded == HANDLE_PRESENT) {
        const handleInfo = decoder.claimHandle()
        return checkHandleRights(handleInfo, objectType, rights)
    }

    return null
}

function decodeHandle(decoder: Decoder, offset: number, objectType: number, rights: number): fiber.Handle {
    const handle = decodeNullableHandle(decoder, offset, objectType, rights)
    if (handle === null) {
        throw new MidlError("Found null for a non-nullable type", ErrorCode.NonNullableTypeWithNullValue)
    }
    return handle
}

// TODO(pascallouis): By having _HandleWrapper exported, we could DRY this code
// by simply having an AbstractHandleType<H extend HandleWrapper<H>> and having
// the encoding / decoding once, with the only specialization on a per-type
// basis being construction.
// Further, if each HandleWrapper were to offer a static ctor function to invoke
// their constrctors, could be called directly.
// We could also explore having a Handle be itself a subtype of HandleWrapper
// to further standardize handling of handles.
abstract class _BaseHandleType<W> extends SimpleMidlType<W> {
    constructor(private readonly objectType: number, private readonly rights: number) {
        super(4)
    }

    abstract wrap(handle: fiber.Handle): W
    abstract unwrap(wrapper: W): fiber.Handle | null

    private asHandleDisposition(value: W): fiber.HandleDisposition | null {
        const handle = this.unwrap(value)
        if (handle !== null) {
            return new fiber.HandleDisposition(FX_HANDLE_OP_MOVE, handle, this.objectType, this.rights)
        }
        return null
    }

    override encode(encoder: Encoder, value: W, offset: number, depth: number) {
        encodeHandle(encoder, this.asHandleDisposition(value), offset, false)
    }

    override decode(decoder: Decoder, offset: number, depth: number) {
        return this.wrap(decodeHandle(decoder, offset, this.objectType, this.rights))
    }
}

export class HandleType extends _BaseHandleType<fiber.Handle> {
    constructor(objectType: number, rights: number) {
        super(objectType, rights)
    }

    override wrap(handle: fiber.Handle): fiber.Handle {
        return handle
    }

    override unwrap(handle: fiber.Handle): fiber.Handle | null {
        return handle
    }
}

export class ChannelType extends _BaseHandleType<fiber.Channel> {
    constructor(objectType: number, rights: number) {
        super(objectType, rights)
    }

    override wrap(handle: fiber.Handle): fiber.Channel {
        return new fiber.Channel(handle)
    }

    override unwrap(wrapper: fiber.Channel): fiber.Handle | null {
        return wrapper.handle
    }
}

/*export class SocketType extends _BaseHandleType<fiber.Socket> {
    constructor(objectType: number, rights: number) {
        super(objectType, rights)
    }

    override wrap(handle: fiber.Handle): fiber.Socket {
        return new fiber.Channel(handle)
    }

    override unwrap(wrapper: fiber.Socket): fiber.Handle | null {
        return wrapper.handle
    }
}*/

export class InterfaceHandleType<T> extends SimpleMidlType<InterfaceHandle<T>> {
    constructor() {
        super(4)
    }

    override encode(encoder: Encoder, value: InterfaceHandle<T>, offset: number, depth: number) {
        const handle = value.channel?.handle
        encodeHandle(
            encoder,
            handle == null
                ? null
                : new fiber.HandleDisposition(
                      FX_HANDLE_OP_MOVE,
                      handle,
                      FX_OBJ_TYPE_CHANNEL,
                      FX_DEFAULT_CHANNEL_RIGHTS
                  ),
            offset,
            false
        )
    }

    override decode(decoder: Decoder, offset: number, depth: number): InterfaceHandle<T> {
        return new InterfaceHandle<T>(
            new fiber.Channel(decodeHandle(decoder, offset, FX_OBJ_TYPE_CHANNEL, FX_DEFAULT_CHANNEL_RIGHTS))
        )
    }
}

export class InterfaceRequestType<T> extends SimpleMidlType<InterfaceRequest<T>> {
    constructor() {
        super(4)
    }

    override encode(encoder: Encoder, value: InterfaceRequest<T>, offset: number, depth: number) {
        const handle = value.channel?.handle
        encodeHandle(
            encoder,
            handle == null
                ? null
                : new fiber.HandleDisposition(
                      FX_HANDLE_OP_MOVE,
                      handle,
                      FX_OBJ_TYPE_CHANNEL,
                      FX_DEFAULT_CHANNEL_RIGHTS
                  ),
            offset,
            false
        )
    }

    override decode(decoder: Decoder, offset: number, depth: number): InterfaceRequest<T> {
        return new InterfaceRequest<T>(
            new fiber.Channel(decodeHandle(decoder, offset, FX_OBJ_TYPE_CHANNEL, FX_DEFAULT_CHANNEL_RIGHTS))
        )
    }
}

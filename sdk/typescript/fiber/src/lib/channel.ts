import { fx_channel_create } from "@meshx-org/fiber-sys"
import {
    FX_INVALID_HANDLE,
    FX_CHANNEL_READ_MAY_DISCARD,
    FX_CHANNEL_MAX_MSG_BYTES,
    FX_CHANNEL_MAX_MSG_HANDLES,
    FX_CHANNEL_PEER_CLOSED,
    FX_CHANNEL_READABLE,
    FX_CHANNEL_WRITABLE,
    Ref,
    Status,
} from "@meshx-org/fiber-types"
import { Handle } from "./handle"
import { HandleWrapper, HandleWrapperPair } from "./handleWrapper"

/// Typed wrapper around a Zircon channel object.
export class Channel extends HandleWrapper {
    constructor(handle: Handle | null) {
        super(handle)
    }

    // Signals
    static READABLE = FX_CHANNEL_READABLE
    static WRITABLE = FX_CHANNEL_WRITABLE
    static PEER_CLOSED = FX_CHANNEL_PEER_CLOSED

    // Read options
    static READ_MAY_DISCARD = FX_CHANNEL_READ_MAY_DISCARD

    // Limits
    static MAX_MSG_BYTES = FX_CHANNEL_MAX_MSG_BYTES
    static MAX_MSG_HANDLES = FX_CHANNEL_MAX_MSG_HANDLES

    /* 
    int write(ByteData data, [List<Handle>? handles]) {
        if (handle == null) {
        return ZX.ERR_INVALID_ARGS;
        }
        return System.channelWrite(handle!, data, handles ?? []);
    }

    int writeEtc(ByteData data, [List<HandleDisposition>? handleDispositions]) {
        if (handle == null) {
        return ZX.ERR_INVALID_ARGS;
        }
        return System.channelWriteEtc(handle!, data, handleDispositions ?? []);
    }

    ReadResult queryAndRead() {
        if (handle == null) {
        return const ReadResult(ZX.ERR_INVALID_ARGS);
        }
        return System.channelQueryAndRead(handle!);
    }

    ReadEtcResult queryAndReadEtc() {
        if (handle == null) {
        return const ReadEtcResult(ZX.ERR_INVALID_ARGS);
        }
        return System.channelQueryAndReadEtc(handle!);
    }
    */
}

/// Typed wrapper around a linked pair of channel objects and the
/// fx_channel_create() syscall used to create them.
export class ChannelPair extends HandleWrapperPair<Channel> {
    static create(): ChannelPair {
        const first = new Ref(FX_INVALID_HANDLE)
        const second = new Ref(FX_INVALID_HANDLE)
        const status = fx_channel_create(first, second)

        if (status === Status.OK) {
            const firstChannel = new Channel(new Handle(first.value))
            const secondChannel = new Channel(new Handle(first.value))
            return new ChannelPair(status, firstChannel, secondChannel)
        }

        return new ChannelPair(status, null, null)
    }

    private constructor(
        status: Status,
        first: Channel | null,
        second: Channel | null
    ) {
        super(status, first, second)
    }
}

import {
    FX_CHANNEL_MAX_MSG_BYTES,
    FX_CHANNEL_MAX_MSG_HANDLES,
    FX_CHANNEL_PEER_CLOSED,
    FX_CHANNEL_READABLE,
    FX_CHANNEL_READ_MAY_DISCARD,
    FX_CHANNEL_WRITABLE,
    FX_HANDLE_OP_DUPLICATE,
    FX_HANDLE_INVALID,
    Ref,
    Status,
    fx_handle_disposition_t,
    u32,
    fx_handle_t,
} from "@meshx-org/fiber-types";
import { HandleWrapper, HandleWrapperPair } from "./handle-wrapper";
import { Handle } from "./handle";
import { CreateResult, HandleDisposition, ReadEtcResult, ReadResult } from "./types";

/// Typed wrapper around a linked pair of channel objects and the
/// fx_channel_create() syscall used to create them.
export class Channel extends HandleWrapper {
    constructor(handle: Handle) {
        super(handle);
    }

    // Signals
    static READABLE = FX_CHANNEL_READABLE;
    static WRITABLE = FX_CHANNEL_WRITABLE;
    static PEER_CLOSED = FX_CHANNEL_PEER_CLOSED;

    // Read options
    static READ_MAY_DISCARD = FX_CHANNEL_READ_MAY_DISCARD;

    // Limits
    static MAX_MSG_BYTES = FX_CHANNEL_MAX_MSG_BYTES;
    static MAX_MSG_HANDLES = FX_CHANNEL_MAX_MSG_HANDLES;

    /*writeEtc(data: Uint8Array, handleDispositions: HandleDisposition[] = []): Status {
        if (!this.handle || !this.handle.isValid) return Status.ERR_BAD_HANDLE;

        const fx_handle_dispositions: fx_handle_disposition_t[] = [];
        for (const handle of handleDispositions) {
            // FML_DCHECK(handle->result() == ZX_OK);
            fx_handle_dispositions.push({
                operation: handle.operation,
                handle: handle.handle.raw,
                type: handle.type,
                rights: handle.rights,
                result: Status.OK,
            });
        }

        const status = self.fiber.sys_channel_write_etc(
            this.handle.raw,
            0,
            data.buffer,
            data.byteLength,
            fx_handle_dispositions,
            fx_handle_dispositions.length
        );

        for (let i = 0; i < handleDispositions.length; ++i) {
            handleDispositions[i].result = fx_handle_dispositions[i].result;

            // Handles that are not copied (i.e. moved) are always consumed.
            if (handleDispositions[i].operation != FX_HANDLE_OP_DUPLICATE) {
                handleDispositions[i].handle = Handle.invalid();
            }
        }

        return status;
    }*/

    write(flags: u32, data: Uint8Array, handles: fx_handle_t[]): Status {
        if (!this.handle || !this.handle.isValid) return Status.ERR_BAD_HANDLE;

        const status = self.fiber.sys_channel_write(this.handle.raw, 0, data, handles);

        return status;
    }

    read(flags: u32, num_bytes: u32, num_handles: u32): ReadResult {
        const bytes = { pointee: new Uint8Array() };
        const handles = { pointee: [] };
        const actual_bytes = { pointee: 0 };
        const actual_handles = { pointee: 0 };

        const status = self.fiber.sys_channel_read(
            this.handle.raw,
            0,
            bytes,
            handles,
            num_bytes,
            num_handles,
            actual_bytes,
            actual_handles
        );

        return {
            status,
            handles: handles.pointee.map((raw) => Handle.from_raw(raw)),
            bytes: bytes.pointee,
            numBytes: actual_bytes.pointee,
        };
    }

    public queryAndReadEtc(): ReadEtcResult {
        return {
            status: Status.OK,
            bytes: new Uint8Array(),
            handleInfos: [],
        };
    }
}

/// Typed wrapper around a linked pair of channel objects and the
/// fx_channel_create() syscall used to create them.
export class ChannelPair extends HandleWrapperPair<Channel> {
    static create(): CreateResult<ChannelPair> {
        const first = new Ref(FX_HANDLE_INVALID);
        const second = new Ref(FX_HANDLE_INVALID);
        const status = self.fiber.sys_channel_create(0, first, second);

        if (status !== Status.OK) {
            return { status };
        }

        const firstChannel = new Channel(Handle.from_raw(first.value));
        const secondChannel = new Channel(Handle.from_raw(second.value));

        return {
            status,
            handle: new ChannelPair(firstChannel, secondChannel),
        };
    }

    private constructor(first: Channel, second: Channel) {
        super(first, second);
    }
}

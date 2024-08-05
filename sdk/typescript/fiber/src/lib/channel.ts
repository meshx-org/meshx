import {
    fx_channel_create,
    fx_channel_read,
    fx_channel_read_etc,
    fx_channel_write,
    fx_channel_write_etc,
} from "@meshx-org/fiber-sys"
import {
    FX_HANDLE_INVALID,
    FX_CHANNEL_READ_MAY_DISCARD,
    FX_CHANNEL_MAX_MSG_BYTES,
    FX_CHANNEL_MAX_MSG_HANDLES,
    FX_CHANNEL_PEER_CLOSED,
    FX_CHANNEL_READABLE,
    FX_CHANNEL_WRITABLE,
    FX_HANDLE_OP_DUPLICATE,
    Ref,
    Status,
    fx_handle_t,
    fx_handle_disposition_t,
    fx_handle_info_t,
} from "@meshx-org/fiber-types"
import { Handle } from "./handle"
import { HandleWrapper, HandleWrapperPair } from "./handleWrapper"
import { HandleDisposition, ReadEtcResult, ReadResult } from "./types"

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

    public queryAndRead(): ReadResult {
        if (!this.handle || !this.handle.isValid) {
            return { status: Status.ERR_BAD_HANDLE }
        }

        const actualBytes = new Ref(0)
        const actualHandles = new Ref(0)

        // Query the size of the next message.
        let status = fx_channel_read(this.handle.raw, 0, null, null, 0, 0, actualBytes, actualHandles)

        if (status != Status.ERR_BUFFER_TOO_SMALL) {
            // An empty message or an error.
            return { status }
        }

        // Allocate space for the bytes and handles.
        const bytes = new DataView(new ArrayBuffer(actualBytes.value))
        const handles: Array<fx_handle_t> = new Array(actualHandles.value)

        // Make the call to actually get the message.
        status = fx_channel_read(
            this.handle.raw,
            0,
            bytes.buffer,
            handles,
            bytes.byteLength,
            handles.length,
            actualBytes,
            actualHandles
        )
        // FML_DCHECK(status != ZX_OK || bytes.size() == actual_bytes);

        if (status == Status.OK) {
            // FML_DCHECK(handles.size() == actual_handles);

            // return a ReadResult object.
            return {
                status,
                bytes: bytes,
                numBytes: actualBytes.take(),
                handles: handles.map((h) => new Handle(h)),
            }
        } else {
            return { status }
        }
    }

    write(data: ArrayBuffer | undefined, handles: Handle[] = []): Status {
        if (!this.handle || !this.handle.isValid) return Status.ERR_BAD_HANDLE

        const fx_handles: Array<fx_handle_t> = []
        for (const handle of handles) {
            fx_handles.push(handle.raw)
        }

        const status = fx_channel_write(this.handle.raw, 0, data!, data!.byteLength, fx_handles, fx_handles.length)

        // Handles are always consumed.
        for (let h of handles) {
            h = Handle.invalid()
        }

        data = undefined
        return status
    }

    writeEtc(data: DataView | undefined, handleDispositions: HandleDisposition[] = []): Status {
        if (!this.handle || !this.handle.isValid) return Status.ERR_BAD_HANDLE

        const fx_handle_dispositions: fx_handle_disposition_t[] = []
        for (const handle of handleDispositions) {
            // FML_DCHECK(handle->result() == ZX_OK);
            fx_handle_dispositions.push({
                operation: handle.operation,
                handle: handle.handle!.raw,
                type: handle.type,
                rights: handle.rights,
                result: Status.OK,
            })
        }

        const status = fx_channel_write_etc(
            this.handle.raw,
            0,
            data!.buffer,
            data!.byteLength,
            fx_handle_dispositions,
            fx_handle_dispositions.length
        )

        for (let i = 0; i < handleDispositions.length; ++i) {
            handleDispositions[i].result = fx_handle_dispositions[i].result

            // Handles that are not copied (i.e. moved) are always consumed.
            if (handleDispositions[i].operation != FX_HANDLE_OP_DUPLICATE) {
                handleDispositions[i].handle = undefined
            }
        }

        data = undefined
        return status
    }

    queryAndReadEtc(): ReadEtcResult {
        if (!this.handle || !this.handle.isValid) {
            return { status: Status.ERR_BAD_HANDLE }
        }

        const actual_bytes = new Ref(0)
        const actual_handles = new Ref(0)

        // Query the size of the next message.
        let status = fx_channel_read(this.handle.raw, 0, null, null, 0, 0, actual_bytes, actual_handles)
        if (status != Status.ERR_BUFFER_TOO_SMALL) {
            // An empty message or an error.
            return { status }
        }

        // Allocate space for the bytes and handles.
        let bytes: ArrayBuffer | undefined = new ArrayBuffer(actual_bytes.value)
        const handles: Array<fx_handle_info_t> = new Array(actual_handles.value)

        // Make the call to actually get the message.
        status = fx_channel_read_etc(
            this.handle.raw,
            0,
            bytes,
            handles,
            bytes.byteLength,
            handles.length,
            actual_bytes,
            actual_handles
        )
        // FML_DCHECK(status != ZX_OK || bytes.size() == actual_bytes);

        bytes = undefined

        if (status === Status.OK) {
            // TODO: FML_DCHECK(handles.size() == actual_handles);

            // return a ReadResult object.
            return {
                status,
                bytes: bytes,
                numBytes: actual_bytes.take(),
                handleInfos: handles.map((h) => ({
                    ...h,
                    handle: new Handle(h.handle),
                })),
            }
        } else {
            return { status }
        }
    }
}

/// Typed wrapper around a linked pair of channel objects and the
/// fx_channel_create() syscall used to create them.
export class ChannelPair extends HandleWrapperPair<Channel> {
    static create(): ChannelPair {
        const first = new Ref(FX_HANDLE_INVALID)
        const second = new Ref(FX_HANDLE_INVALID)
        const status = fx_channel_create(0, first, second)

        if (status === Status.OK) {
            const firstChannel = new Channel(new Handle(first.value))
            const secondChannel = new Channel(new Handle(first.value))
            return new ChannelPair(status, firstChannel, secondChannel)
        }

        return new ChannelPair(status, null, null)
    }

    private constructor(status: Status, first: Channel | null, second: Channel | null) {
        super(status, first, second)
    }
}

// Definition of a MessagePacket's specific pointer type. Message packets must
// be managed using this specific type of pointer, because MessagePackets have a

import { FX_ERR_BUFFER_TOO_SMALL, FX_ERR_OUT_OF_RANGE, FX_OK, fx_status_t, fx_txid_t } from "@meshx-org/fiber-types"
import { Handle } from "./handle"
import { Err, Ok, Result, user_ptr } from "../std"

// specific custom deletion requirement.
export type MessagePacketPtr = MessagePacket

// Handles are stored just after the MessagePacket.
// const HANDLES_OFFSET: number = std::mem::size_of::<MessagePacket>() as u32;

export const MAX_MESSAGE_SIZE = 65536
export const MAX_MESSAGE_HANDLES = 64

// ensure public constants are aligned
//assert.equal(FX_CHANNEL_MAX_MSG_BYTES, MAX_MESSAGE_SIZE)
//assert.equal(FX_CHANNEL_MAX_MSG_HANDLES, MAX_MESSAGE_HANDLES)

// PayloadOffset returns the offset of the data payload from the start of the first buffer.
function payload_offset(num_handles: number): number {
    // The payload comes after the handles.
    return 0 // TODO HANDLES_OFFSET + num_handles * std::mem::size_of::<*const Handle>() as u32;
}

export class MessagePacket {
    #data_size: number
    #num_handles: number
    #owns_handles: boolean
    public data: Uint8Array
    #data_view: DataView
    public handles: (Handle | null)[]

    // A private constructor ensures that users must use the static factory
    // Create method to create a MessagePacket. This, in turn, guarantees that
    // when a user creates a MessagePacket, they end up with the proper
    // MessagePacket::UPtr type for managing the message packet's life cycle.
    constructor(data: Uint8Array, data_size: number, num_handles: number, handles: Handle[]) {
        this.data = data
        this.#data_view = new DataView(data.buffer)
        this.handles = handles
        this.#data_size = data_size
        this.#num_handles = num_handles
        this.#owns_handles = false
    }

    // Creates a MessagePacket in |msg| sufficient to hold |data_size| bytes and |num_handles|.
    //
    // Note: This method does not write the payload into the MessagePacket.
    //
    // Returns FX_OK on success.
    static create_common(init_data: Uint8Array | null, data_size: number, num_handles: number): Result<MessagePacketPtr, fx_status_t> {
        if ((data_size as number) > MAX_MESSAGE_SIZE || (num_handles as number) > MAX_MESSAGE_HANDLES) {
            return Err(FX_ERR_OUT_OF_RANGE)
        }

        // const payload_offset = payload_offset(num_handles as number)

        // MessagePackets lives *inside* a list of buffers. The first buffer holds the MessagePacket
        // object, followed by its handles (if any), and finally the payload data.
        const data = init_data ?? new Uint8Array(data_size)

        const handles = new Array(MAX_MESSAGE_HANDLES).fill(null)
        handles.length = num_handles

        // Construct the MessagePacket into the first buffer.
        // assert(MAX_MESSAGE_HANDLES <= UINT16_MAX, "");
        const msg = new MessagePacket(data, data_size, num_handles, handles)

        // The MessagePacket now owns the BufferChain and msg owns the MessagePacket.
        return Ok(msg)
    }

    // Creates a message packet containing the provided data and space for
    // |num_handles| handles. The handles array is uninitialized and must
    // be completely overwritten by clients.
    static create(
        data: Uint8Array | null,
        data_size: number,
        num_handles: number
    ): Result<MessagePacketPtr, fx_status_t> {
        const result = MessagePacket.create_common(data, data_size, num_handles)

        if (!result.ok) {
            return result
        }

        return Ok(result.value)
    }

    // Copies the packet's |data_size()| bytes to |buf|.
    // Returns an error if |buf| points to a bad user address.
    copy_data_to(buf: user_ptr<Uint8Array>): fx_status_t {
        if (buf.pointee.byteLength < this.#data_size) {
            console.log("copy_data_to", "FX_ERR_BUFFER_TOO_SMALL", buf.pointee.byteLength, this.#data_size)
            return FX_ERR_BUFFER_TOO_SMALL
        }

        buf.pointee.set(this.data)
        return FX_OK
    }

    data_size(): number {
        return this.#data_size
    }

    num_handles(): number {
        return this.#num_handles
    }

    set_owns_handles(own_handles: boolean) {
        this.#owns_handles = own_handles
    }

    // fx_channel_call treats the leading bytes of the payload as
    // a transaction id of type fx_txid_t.
    get_txid(): fx_txid_t {
        if (this.#data_size < 4) {
            return 0
        }

        // The first few bytes of the payload are the fx_txid_t.
        return this.#data_view.getUint32(0) as fx_txid_t
    }

    set_txid(txid: fx_txid_t) {
        if (this.#data_size >= 4) {
            this.#data_view.setUint32(0, txid)
        }
    }
}

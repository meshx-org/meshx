use super::Handle;
use fiber_sys as sys;
use static_assertions::const_assert;

// Definition of a MessagePacket's specific pointer type. Message packets must
// be managed using this specific type of pointer, because MessagePackets have a
// specific custom deletion requirement.
pub(crate) type MessagePacketPtr = Box<MessagePacket>;

// Handles are stored just after the MessagePacket.
const HANDLES_OFFSET: u32 = std::mem::size_of::<MessagePacket>() as u32;

const MAX_MESSAGE_SIZE: u32 = 65536;
const MAX_MESSAGE_HANDLES: u32 = 64;

// ensure public constants are aligned
const_assert!(sys::FX_CHANNEL_MAX_MSG_BYTES == MAX_MESSAGE_SIZE);
const_assert!(sys::FX_CHANNEL_MAX_MSG_HANDLES == MAX_MESSAGE_HANDLES);

// PayloadOffset returns the offset of the data payload from the start of the first buffer.
const fn payload_offset(num_handles: u32) -> u32 {
    // The payload comes after the handles.
    return HANDLES_OFFSET + num_handles * std::mem::size_of::<*const Handle>() as u32;
}

pub(crate) struct MessagePacket {
    data_size: usize,
    num_handles: u16,
    payload_offset: usize,
    buffer_chain: BufferChain,
    owns_handles: bool,
    handles: Vec<Handle>,
    // TODO: handles
}

impl MessagePacket {
    // A private constructor ensures that users must use the static factory
    // Create method to create a MessagePacket.  This, in turn, guarantees that
    // when a user creates a MessagePacket, they end up with the proper
    // MessagePacket::UPtr type for managing the message packet's life cycle.
    fn new(
        buffer_chain: *const BufferChain,
        data_size: usize,
        payload_offset: usize,
        num_handles: u16,
        handles: Vec<Handle>,
    ) -> Self {
        MessagePacket {
            buffer_chain,
            handles,
            data_size,
            payload_offset,
            num_handles,
            owns_handles: false,
        }
    }

    // Creates a MessagePacket in |msg| sufficient to hold |data_size| bytes and |num_handles|.
    //
    // Note: This method does not write the payload into the MessagePacket.
    //
    // Returns FX_OK on success.
    fn create_common(data_size: usize, num_handles: u16) -> Result<MessagePacketPtr, sys::fx_status_t> {
        if data_size > kMaxMessageSize || num_handles > kMaxMessageHandles {
            return Err(sys::FX_ERR_OUT_OF_RANGE);
        }

        let payload_offset = payload_offset(num_handles as u32);

        // MessagePackets lives *inside* a list of buffers.  The first buffer holds the MessagePacket
        // object, followed by its handles (if any), and finally the payload data.
        let chain = BufferChain::alloc(payload_offset + data_size as u32);
        if !chain {
            return Err(sys::FX_ERR_NO_MEMORY);
        }

        debug_assert!(!chain.buffers().is_empty());
        chain.skip(payload_offset);

        let data = chain.buffers().front().data();
        let handles = data + HANDLES_OFFSET;

        // Construct the MessagePacket into the first buffer.
        // static_assert(kMaxMessageHandles <= UINT16_MAX, "");
        let msg = Box::new(MessagePacket::new(
            chain,
            data_size,
            payload_offset,
            num_handles,
            handles,
        ));

        // The MessagePacket now owns the BufferChain and msg owns the MessagePacket.
        return Ok(msg);
    }

    // Creates a message packet containing the provided data and space for
    // |num_handles| handles. The handles array is uninitialized and must
    // be completely overwritten by clients.
    pub(crate) fn create(
        data: *const u8,
        data_size: usize,
        num_handles: u16,
    ) -> Result<MessagePacketPtr, sys::fx_status_t> {
        let result = MessagePacket::create_common(data_size, num_handles);

        if result.is_err() {
            return result;
        }

        let new_msg = result.unwrap();
        let result = new_msg.buffer_chain.append_kernel(data, data_size);

        if result.is_err() {
            return result;
        }

        Ok(new_msg)
    }

    // Copies the packet's |data_size()| bytes to |buf|.
    // Returns an error if |buf| points to a bad user address.
    pub(crate) fn copy_data_to(&self, buf: &mut [u8]) -> sys::fx_status_t {
        return self.buffer_chain.CopyOut(buf, self.payload_offset, self.data_size);
    }

    pub(crate) fn data_size(&self) -> usize {
        self.data_size
    }

    pub(crate) fn num_handles(&self) -> u16 {
        return self.num_handles;
    }

    pub(crate) fn handles(&self) -> *const Handle {
        self.handles.as_ptr()
    }

    pub(crate) fn mutable_handles(&self) -> &mut Vec<Handle> {
        self.handles.as_mut()
    }

    fn set_owns_handles(&self, own_handles: bool) {
        self.owns_handles = own_handles;
    }

    // fx_channel_call treats the leading bytes of the payload as
    // a transaction id of type fx_txid_t.
    pub(crate) fn get_txid(&self) -> sys::fx_txid_t {
        if self.data_size < std::mem::size_of::<sys::fx_txid_t>() {
            return 0;
        }

        // The first few bytes of the payload are a zx_txid_t.
        let payload_start = self.buffer_chain.buffers().front().data() + self.payload_offset;
        return (payload_start) as sys::fx_txid_t;
    }

    pub(crate) fn set_txid(&self, txid: sys::fx_txid_t) {
        if self.data_size >= std::mem::size_of::<sys::fx_txid_t>() {
            let payload_start = self.buffer_chain.buffers().front().data() + self.payload_offset;
            *payload_start = txid;
        }
    }
}

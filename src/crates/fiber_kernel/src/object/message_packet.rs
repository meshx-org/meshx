use std::sync::Weak;

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

#[derive(Debug, Default)]
pub(crate) struct MessagePacket {
    data_size: usize,
    num_handles: u32,
    owns_handles: bool,
    data: Vec<u8>,
    handles: Vec<Option<Weak<Handle>>>,
}

impl MessagePacket {
    // A private constructor ensures that users must use the static factory
    // Create method to create a MessagePacket.  This, in turn, guarantees that
    // when a user creates a MessagePacket, they end up with the proper
    // MessagePacket::UPtr type for managing the message packet's life cycle.
    fn new(data: Vec<u8>, data_size: usize, num_handles: u32, handles: Vec<Option<Weak<Handle>>>) -> Self {
        MessagePacket {
            data,
            handles,
            data_size,
            num_handles,
            owns_handles: false,
        }
    }

    // Creates a MessagePacket in |msg| sufficient to hold |data_size| bytes and |num_handles|.
    //
    // Note: This method does not write the payload into the MessagePacket.
    //
    // Returns FX_OK on success.
    fn create_common(data_size: usize, num_handles: u32) -> Result<MessagePacketPtr, sys::fx_status_t> {
        if data_size as u32 > MAX_MESSAGE_SIZE || num_handles > MAX_MESSAGE_HANDLES {
            return Err(sys::FX_ERR_OUT_OF_RANGE);
        }

        let payload_offset = payload_offset(num_handles);

        // MessagePackets lives *inside* a list of buffers. The first buffer holds the MessagePacket
        // object, followed by its handles (if any), and finally the payload data.
        let data = Vec::with_capacity(data_size);

        let mut handles = Vec::with_capacity(MAX_MESSAGE_HANDLES as usize);
        handles.resize_with(num_handles as usize, || None); 

        // Construct the MessagePacket into the first buffer.
        // static_assert(kMaxMessageHandles <= UINT16_MAX, "");
        let msg = Box::new(MessagePacket::new(data, data_size, num_handles, handles));

        // The MessagePacket now owns the BufferChain and msg owns the MessagePacket.
        return Ok(msg);
    }

    // Creates a message packet containing the provided data and space for
    // |num_handles| handles. The handles array is uninitialized and must
    // be completely overwritten by clients.
    pub(crate) fn create(
        data: *const u8,
        data_size: usize,
        num_handles: u32,
    ) -> Result<MessagePacketPtr, sys::fx_status_t> {
        let result = MessagePacket::create_common(data_size, num_handles);

        if result.is_err() {
            return result;
        }

        Ok(result.unwrap())
    }

    // Copies the packet's |data_size()| bytes to |buf|.
    // Returns an error if |buf| points to a bad user address.
    pub(crate) fn copy_data_to(&self, buf: &mut [u8]) -> sys::fx_status_t {
        if buf.len() < self.data_size {
            return sys::FX_ERR_BUFFER_TOO_SMALL;
        }

        buf[..self.data_size].copy_from_slice(&self.data[..self.data_size]);

        return sys::FX_OK;
    }

    pub(crate) fn data_size(&self) -> usize {
        self.data_size
    }

    pub(crate) fn num_handles(&self) -> u32 {
        return self.num_handles;
    }

    pub(crate) fn handles(&self) -> &Vec<Option<Weak<Handle>>> {
        self.handles.as_ref()
    }

    pub(crate) fn mutable_handles(&mut self) -> &mut Vec<Option<Weak<Handle>>> {
        self.handles.as_mut()
    }

    fn set_owns_handles(&mut self, own_handles: bool) {
        self.owns_handles = own_handles;
    }

    // fx_channel_call treats the leading bytes of the payload as
    // a transaction id of type fx_txid_t.
    pub(crate) fn get_txid(&self) -> sys::fx_txid_t {
        if self.data_size < std::mem::size_of::<sys::fx_txid_t>() {
            return 0;
        }

        // The first few bytes of the payload are a zx_txid_t.
        let payload_start = self.data.as_ptr();
        return (payload_start) as sys::fx_txid_t;
    }

    pub(crate) fn set_txid(&mut self, txid: sys::fx_txid_t) {
        if self.data_size >= std::mem::size_of::<sys::fx_txid_t>() {
            let payload_start = self.data.as_mut_ptr();
            unsafe {
                *payload_start = txid as u8;
            }
        }
    }
}

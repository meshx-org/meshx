#![allow(unused)] 

use serde::{Deserialize, Serialize};

pub mod helpers;
pub mod ir;

#[derive(Debug, Serialize, Deserialize)]
pub struct PaddingMarker {
    // Offset into the struct in bytes (0 is the start of the struct).
    offset: u32,
    // Width of the mask in bits. Either 16, 32, or 64.
    mask_bit_width: u32,
    // Little endian mask with 0x00 for non-padding and 0xff for padding.
    // Only the lower MaskBitWidth bits are used. The higher bits are 0.
    mask: u64,
}

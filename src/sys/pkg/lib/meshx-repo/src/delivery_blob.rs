/// Type of delivery blob.
///
/// **WARNING**: These constants are used when generating delivery blobs and should not be changed.
/// Non backwards-compatible changes to delivery blob formats should be made by creating a new type.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
#[repr(u32)]
pub enum DeliveryBlobType {
    /// Reserved for internal use.
    Reserved = 0,
    /// Type 1 delivery blobs support the zstd-chunked compression format.
    Type1 = 1,
}

/// Generate a delivery blob of the specified `delivery_type` for `data` using default parameters
/// and write the generated blob to `writer`.
pub fn generate_to(
    delivery_type: DeliveryBlobType,
    data: &[u8],
    writer: impl std::io::Write,
) -> Result<(), std::io::Error> {
    match delivery_type {
        DeliveryBlobType::Type1 => todo!(),
        _ => panic!("Unsupported delivery blob type: {:?}", delivery_type),
    }
}
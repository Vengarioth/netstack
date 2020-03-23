pub const HEADER_SIZE: usize = std::mem::size_of::<Header>();

#[repr(C, packed)]
pub struct Header {
    pub hmac: [u8; 32],
    pub sequence_number: u64,
    pub ack_sequence_number: u64,
    pub ack_bits: [u8; 4],
    pub packet_type: u8,
    pub padding: u8,
    pub body_length: u16,
}

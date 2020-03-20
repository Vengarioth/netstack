use sha2::Sha256;
use hmac::{Hmac, Mac};

pub const MTU: usize = 1500;
pub const HEADER_SIZE: usize = std::mem::size_of::<Header>();
pub const HEADER_HMAC_SIZE: usize = 32;
pub type Buffer = [u8; MTU];
type HmacSha256 = Hmac<Sha256>;

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

pub struct Packet {
    buffer: Buffer,
}

impl Packet {
    pub fn new() -> Self {
        Self {
            buffer: [0; MTU],
        }
    }

    pub fn from_buffer(buffer: Buffer) -> Self {
        Self {
            buffer,
        }
    }

    pub fn get_buffer(&self) -> &Buffer {
        &self.buffer
    }

    pub fn get_buffer_mut(&mut self) -> &mut Buffer {
        &mut self.buffer
    }

    pub fn get_header<'a>(&'a self) -> &'a Header {
        use std::mem::transmute;

        let ptr = &self.buffer as *const _;
        let reference = unsafe { transmute(ptr) };
        reference
    }

    pub fn get_header_mut<'a>(&'a mut self) -> &'a mut Header {
        use std::mem::transmute;

        let ptr = &mut self.buffer as *mut _;
        let reference = unsafe { transmute(ptr) };
        reference
    }

    pub fn get_slice(&self, start: usize, end: usize) -> &[u8] {
        &self.buffer[start..end]
    }
    
    pub fn get_slice_mut(&mut self, start: usize, end: usize) -> &mut [u8] {
        &mut self.buffer[start..end]
    }

    pub fn sign(&mut self, key: &[u8]) {
        let mut mac = HmacSha256::new_varkey(key).expect("HMac can take a key of any size");
        mac.input(&self.buffer[HEADER_HMAC_SIZE..]);
        let result = mac.result().code();

        dbg!(result.len());

        for i in 0..HEADER_HMAC_SIZE {
            self.buffer[i] = result[i];
        }
    }

    pub fn verify_signature(&self, key: &[u8]) -> bool {
        let mut mac = HmacSha256::new_varkey(key).expect("HMAC can take a key of any size");
        mac.input(&self.buffer[HEADER_HMAC_SIZE..]);
        mac.verify(&self.buffer[0..HEADER_HMAC_SIZE]).is_ok()
    }

    pub fn into_buffer(self) -> Buffer {
        self.buffer
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_set_data_in_header() {
        let mut packet = Packet::new();

        let header = packet.get_header_mut();
        header.hmac[0] = 16;

        assert_eq!(packet.get_buffer()[0], 16);
        assert_eq!(packet.get_header().hmac[0], 16);
        assert_eq!(packet.get_slice(0, 1)[0], 16);
    }

    #[test]
    fn assert_header_size_matches_constants() {
        assert_eq!(HEADER_SIZE, 56);
    }

    #[test]
    fn sign_packet() {
        let mut packet = Packet::new();
        let key: [u8; 4] = [0x1, 0x5, 0x2, 0x8];

        packet.sign(&key);

        assert!(packet.verify_signature(&key));
    }
}

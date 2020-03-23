use sha2::Sha256;
use hmac::{Hmac, Mac};
use std::io::{self, Write};
use super::{RawPacket, HEADER_SIZE};
use crate::security::Secret;

type HmacSha256 = Hmac<Sha256>;

pub struct OutgoingPacket {
    buffer: [u8; 1500],
    bytes_written: usize,
}

impl OutgoingPacket {
    pub fn new() -> Self {
        Self {
            buffer: [0; 1500],
            bytes_written: HEADER_SIZE,
        }
    }

    pub(crate) fn write_header_and_sign(self, sequence_number: u64, ack_sequence_number: u64, ack_bits: [u8; 4], packet_type: u8, secret: &Secret) -> RawPacket {
        let bytes_written = self.bytes_written;
        let mut packet = RawPacket::new(self.buffer, bytes_written);
        let mut header = packet.get_header_mut();
        
        header.sequence_number = sequence_number;
        header.ack_sequence_number = ack_sequence_number;
        header.ack_bits = ack_bits;
        header.packet_type = packet_type;
        header.body_length = (bytes_written - HEADER_SIZE) as u16;

        let mut mac = HmacSha256::new_varkey(secret.get_bytes()).expect("HmacSha256 can take a key of any size");
        mac.input(&packet.get_buffer()[32..bytes_written]);

        let mut header = packet.get_header_mut();
        header.hmac = mac.result().code().into();

        packet
    }
}

impl Write for OutgoingPacket {
    fn write(&mut self, buf: &[u8]) -> Result<usize, std::io::Error> {
        let len = buf.len();
        if self.bytes_written + len > self.buffer.len() {
            panic!("TODO: cannot write, would exceed buffer");
        }

        self.buffer[self.bytes_written..self.bytes_written + len].copy_from_slice(buf);
        self.bytes_written += len;

        Ok(len)
    }
    
    fn flush(&mut self) -> Result<(), io::Error> {
        Ok(())
    }
}

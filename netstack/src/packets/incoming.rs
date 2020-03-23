use std::io::{self, Read};
use super::{RawPacket, HEADER_SIZE, Payload, PacketType};

pub struct IncomingPacket {
    buffer: RawPacket,
    bytes_read: usize,
}

impl IncomingPacket {
    pub fn from_raw_packet(buffer: RawPacket) -> Self {
        Self {
            buffer,
            bytes_read: HEADER_SIZE,
        }
    }

    pub fn get_sequence_number(&self) -> u64 {
        self.buffer.get_header().sequence_number
    }

    pub fn get_ack_sequence_number(&self) -> u64 {
        self.buffer.get_header().ack_sequence_number
    }

    pub fn get_ack_bits(&self) -> [u8; 4] {
        self.buffer.get_header().ack_bits.clone()
    }

    pub fn get_packet_type(&self) -> Option<PacketType> {
        PacketType::from_u8(self.buffer.get_header().packet_type)
    }

    pub fn get_body_length(&self) -> u16 {
        self.buffer.get_header().body_length
    }

    pub fn into_payload(self) -> Payload {
        Payload::from_raw_packet(self.buffer)
    }
}

impl Read for IncomingPacket {
    fn read(&mut self, buf: &mut [u8]) -> Result<usize, io::Error> {
        let len = buf.len();

        if self.bytes_read + len > self.buffer.get_buffer().len() {
            panic!("TODO: cannot read beyond buffer");
        }

        buf.copy_from_slice(&self.buffer.get_buffer()[self.bytes_read..self.bytes_read + len]);
        self.bytes_read += len;

        Ok(len)
    }
}

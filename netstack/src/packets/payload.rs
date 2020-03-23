use super::RawPacket;

pub struct Payload {
    buffer: RawPacket,
}

impl Payload {
    pub fn from_raw_packet(buffer: RawPacket) -> Self {
        Self {
            buffer,
        }
    }

    pub fn get_buffer(&self) -> &[u8] {
        self.buffer.get_body()
    }

    pub fn get_buffer_mut(&mut self) -> &mut [u8] {
        self.buffer.get_body_mut()
    }
}

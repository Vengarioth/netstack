mod raw;
mod header;
mod incoming;
mod outgoing;
mod payload;

pub use raw::*;
pub use header::*;
pub use incoming::*;
pub use outgoing::*;
pub use payload::*;

#[derive(Debug, Eq, PartialEq, Copy, Clone)]
pub enum PacketType {
    Connection,
    Payload,
    Heartbeat,
    Disconnect,
    Disconnected,
}

impl PacketType {
    pub fn from_u8(value: u8) -> Option<Self> {
        match value {
            0 => Some(Self::Connection),
            1 => Some(Self::Payload),
            2 => Some(Self::Heartbeat),
            3 => Some(Self::Disconnect),
            4 => Some(Self::Disconnected),
            _ => None,
        }
    }

    pub fn to_u8(&self) -> u8 {
        match self {
            Self::Connection => 0,
            Self::Payload => 1,
            Self::Heartbeat => 2,
            Self::Disconnect => 3,
            Self::Disconnected => 4,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn sign_and_verify_packet() {
        use std::io::{Read, Write};
        use crate::security::Secret;

        let secret = Secret::from_bytes([
            0x2, 0x1, 0x2, 0x4, 0x8, 0x24, 0x2, 0x1,
            0x2, 0x4, 0x8, 0x24, 0x2, 0x1, 0x2, 0x4,
            0x8, 0x24, 0x2, 0x1, 0x2, 0x4, 0x8, 0x24,
            0x2, 0x1, 0x2, 0x4, 0x8, 0x24, 0x0, 0x64]);
        let mut outgoing = OutgoingPacket::new();

        outgoing.write(&[0x1, 0x2, 0x3, 0x4, 0x5, 0x6]).expect("It writes into the buffer");

        let buffer = outgoing.write_header_and_sign(15, 12, [0x3, 0x2, 0x1, 0x0], 1, &secret);

        let mut incoming = buffer.verify(&secret).expect("The verification succeeds");

        let mut read_into: [u8; 6] = [0; 6];
        incoming.read(&mut read_into).expect("It reads into the buffer");

        assert_eq!(read_into, [0x1, 0x2, 0x3, 0x4, 0x5, 0x6], "The body matches the written data");
        assert_eq!(incoming.get_sequence_number(), 15);
        assert_eq!(incoming.get_ack_sequence_number(), 12);
        assert_eq!(incoming.get_ack_bits(), [0x3, 0x2, 0x1, 0x0]);
        assert_eq!(incoming.get_packet_type(), 1);
        assert_eq!(incoming.get_body_length(), 6);
    }

    #[test]
    fn it_rejects_a_tampered_packet() {
        use std::io::Write;
        use crate::security::Secret;

        let secret = Secret::from_bytes([
            0x2, 0x1, 0x2, 0x4, 0x8, 0x24, 0x2, 0x1,
            0x2, 0x4, 0x8, 0x24, 0x2, 0x1, 0x2, 0x4,
            0x8, 0x24, 0x2, 0x1, 0x2, 0x4, 0x8, 0x24,
            0x2, 0x1, 0x2, 0x4, 0x8, 0x24, 0x0, 0x64]);
        let mut outgoing = OutgoingPacket::new();

        outgoing.write(&[0x1, 0x2, 0x3, 0x4, 0x5, 0x6]).expect("It writes into the buffer");

        let mut buffer = outgoing.write_header_and_sign(0, 1, [0x0, 0x0, 0x0, 0x0], 1, &secret);

        buffer.get_buffer_mut()[56] = 0x2;

        assert!(buffer.verify(&secret).is_none(), "The packet is invalid");
    }

    #[test]
    fn it_rejects_a_packet_signed_with_a_different_secret() {
        use std::io::Write;
        use crate::security::Secret;

        let secret = Secret::from_bytes([
            0x2, 0x1, 0x2, 0x4, 0x8, 0x24, 0x2, 0x1,
            0x2, 0x4, 0x8, 0x24, 0x2, 0x1, 0x2, 0x4,
            0x8, 0x24, 0x2, 0x1, 0x2, 0x4, 0x8, 0x24,
            0x2, 0x1, 0x2, 0x4, 0x8, 0x24, 0x0, 0x64]);
        let mut outgoing = OutgoingPacket::new();

        outgoing.write(&[0x1, 0x2, 0x3, 0x4, 0x5, 0x6]).expect("It writes into the buffer");

        let buffer = outgoing.write_header_and_sign(0, 1, [0x0, 0x0, 0x0, 0x0], 1, &secret);

        let secret = Secret::from_bytes([
            0x5, 0x1, 0x2, 0x4, 0x8, 0x24, 0x2, 0x1,
            0x2, 0x4, 0x8, 0x24, 0x2, 0x1, 0x2, 0x4,
            0x8, 0x24, 0x2, 0x1, 0x2, 0x4, 0x8, 0x24,
            0x2, 0x1, 0x2, 0x4, 0x8, 0x24, 0x0, 0x64]);

        assert!(buffer.verify(&secret).is_none(), "The packet is invalid");
    }
}

use super::{Header, HEADER_SIZE, IncomingPacket};
use crate::security::Secret;

use sha2::Sha256;
use hmac::{Hmac, Mac};
type HmacSha256 = Hmac<Sha256>;

pub struct RawPacket {
    buffer: [u8; 1500],
    length: usize,
}

impl RawPacket {
    pub fn new(buffer: [u8; 1500], length: usize) -> Self {
        Self {
            buffer,
            length,
        }
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

    pub fn get_buffer(&self) -> &[u8] {
        &self.buffer[0..self.length]
    }

    pub fn get_buffer_mut(&mut self) -> &mut [u8] {
        &mut self.buffer[0..self.length]
    }

    pub fn get_body(&self) -> &[u8] {
        &self.buffer[HEADER_SIZE..self.length - HEADER_SIZE]
    }

    pub fn get_body_mut(&mut self) -> &mut [u8] {
        &mut self.buffer[HEADER_SIZE..self.length - HEADER_SIZE]
    }

    pub fn verify(self, secret: &Secret) -> Option<IncomingPacket> {
        let mut mac = HmacSha256::new_varkey(secret.get_bytes()).expect("HmacSha256 can take a key of any size");
        let body_length = self.get_header().body_length as usize;

        mac.input(&self.buffer[32..HEADER_SIZE + body_length]);
        let is_valid = mac.verify(&self.buffer[0..32]).is_ok();

        if is_valid {
            Some(IncomingPacket::from_raw_packet(self))
        } else {
            None
        }
    }
}

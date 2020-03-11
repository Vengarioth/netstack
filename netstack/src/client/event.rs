use std::net::SocketAddr;

pub enum Event {
    Message{
        buffer: [u8; 1500],
        length: usize,
        address: SocketAddr,
    }
}

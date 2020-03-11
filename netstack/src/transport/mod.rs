mod error;
mod udp;
use std::net::SocketAddr;

pub use error::TransportError;
pub use udp::UdpTransport;

pub trait Transport {
    fn poll(&mut self, buffer: &mut [u8]) -> Result<Option<(usize, SocketAddr)>, TransportError>;
    fn send(&mut self, address: &SocketAddr, buffer: &[u8]) -> Result<usize, TransportError>;
}

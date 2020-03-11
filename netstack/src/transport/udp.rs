use super::{Transport, TransportError};
use std::net::{UdpSocket, SocketAddr};
use std::io::ErrorKind;

#[derive(Debug)]
pub struct UdpTransport {
    socket: UdpSocket,
}

impl UdpTransport {
    pub fn new(local_address: SocketAddr) -> Result<Self, TransportError> {
        let socket = UdpSocket::bind(local_address)?;
        socket.set_nonblocking(true)?;

        Ok(Self {
            socket,
        })
    }
}

impl Transport for UdpTransport {
    fn poll(&mut self, buffer: &mut [u8]) -> Result<Option<(usize, SocketAddr)>, TransportError> {
        match self.socket.recv_from(buffer) {
            Ok((amount, source_address)) => Ok(Some((amount, source_address))),
            Err(ref e) if e.kind() == ErrorKind::WouldBlock => Ok(None),
            Err(e) => Err(e.into())
        }
    }

    fn send(&mut self, address: &SocketAddr, buffer: &[u8]) -> Result<usize, TransportError> {
        let amount = self.socket.send_to(buffer, address)?;
        Ok(amount)
    }
}

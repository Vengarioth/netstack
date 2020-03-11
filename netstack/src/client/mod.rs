use super::transport::{Transport, TransportError};
use std::net::SocketAddr;

mod event;
pub use event::Event;

pub struct Client {
    transport: Box<dyn Transport>,
    remote_address: SocketAddr,
}

impl Client {
    pub fn new(transport: Box<dyn Transport>, remote_address: SocketAddr) -> Self {

        Self {
            transport,
            remote_address,
        }
    }

    pub fn update(&mut self) -> Vec<Event> {
        let mut poll_again = true;
        let mut events = Vec::new();
        let mut buffer: [u8; 1500] = [0; 1500];

        while poll_again {
            match self.transport.poll(&mut buffer) {
                Ok(result) => {
                    if let Some((length, address)) = result {
                        events.push(Event::Message {
                            buffer,
                            length,
                            address,
                        });
                        buffer = [0; 1500];
                    } else {
                        poll_again = false;
                    }
                },
                Err(error) => {
                    println!("{}", error);
                }
            }
        }

        events
    }

    pub fn send(&mut self, buffer: &[u8]) -> Result<usize, TransportError> {
        self.transport.send(&self.remote_address, buffer)
    }
}

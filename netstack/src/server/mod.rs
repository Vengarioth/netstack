mod connection;
pub use connection::*;
use super::transport::{Transport, TransportError};
use std::collections::HashMap;
use std::net::SocketAddr;

mod configuration;
pub use configuration::Configuration;

mod event;
pub use event::Event;

pub struct Server {
    transport: Box<dyn Transport>,
    connections: ConnectionList,
    configuration: Configuration,
    address_to_id: HashMap<SocketAddr, Connection>,
    addresses: ConnectionDataList<SocketAddr>,
    timeout: ConnectionDataList<usize>,
}

impl Server {
    pub fn new(configuration: Configuration, transport: Box<dyn Transport>) -> Self {
        let max_connections = configuration.max_connections;
        Self {
            transport,
            connections: ConnectionList::new(max_connections),
            configuration,
            address_to_id: HashMap::new(),
            addresses: ConnectionDataList::new(max_connections),
            timeout: ConnectionDataList::new(max_connections),
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

                        if let Some(connection) = self.address_to_id.get(&address) {
                            // connection exists, reset timeout and add event
                            self.timeout.set(*connection, self.configuration.timeout).unwrap();
                            events.push(Event::Message {
                                connection: connection.clone(),
                                buffer,
                                length,
                            });
                        } else {
                            // add connection, if a slot is free
                            if let Some(connection) = self.connections.create_connection() {
                                self.address_to_id.insert(address, connection);
                                self.timeout.set(connection, self.configuration.timeout).unwrap(); // TODO what to do with errors here?
                                self.addresses.set(connection, address).unwrap();
                                
                                events.push(Event::Connected {
                                    connection: connection.clone(),
                                });
                            }
                        }

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

        let connections: Vec<Connection> = self.connections.into_iter().collect();
        for connection in connections {
            let timeout = self.timeout.get(connection).unwrap().clone();

            if timeout == 0 {
                //connection timeout
                let address = self.addresses.get(connection).unwrap();
                self.address_to_id.remove(address);
                self.connections.delete_connection(connection).unwrap();
                events.push(Event::Disconnected{ connection });
            } else {
                self.timeout.set(connection, timeout - 1).unwrap();
            }
        }

        events
    }

    pub fn send(&mut self, address: &SocketAddr, buffer: &[u8]) -> Result<usize, TransportError> {
        self.transport.send(address, buffer)
    }
}

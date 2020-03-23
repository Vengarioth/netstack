use crate::connection::*;
use super::transport::{Transport, TransportError};
use std::collections::HashMap;
use std::net::SocketAddr;
use crate::packets::{RawPacket, OutgoingPacket};
use crate::security::{Secret, ConnectionToken};

mod configuration;
pub use configuration::Configuration;

mod event;
pub use event::Event;

#[derive(Debug, Eq, PartialEq)]
pub struct ReservedSlot {
    timeout: usize,
    secret: Secret,
    connection_token: ConnectionToken,
}

#[derive(Debug, Eq, PartialEq)]
pub struct ConnectedSlot {
    timeout: usize,
    heartbeat: usize,
    secret: Secret,
    sequence_number: usize,
    address: SocketAddr,
}

#[derive(Debug, Eq, PartialEq)]
pub enum ConnectionSlot {
    Empty,
    Reserved(ReservedSlot),
    Connected(ConnectedSlot),
}

pub struct Server {
    transport: Box<dyn Transport>,
    configuration: Configuration,
    connections: ConnectionList,
    slots: ConnectionDataList<ConnectionSlot>,
    address_to_id: HashMap<SocketAddr, Connection>,
}

impl Server {
    pub fn new(configuration: Configuration, transport: Box<dyn Transport>) -> Self {
        let max_connections = configuration.max_connections;
        Self {
            transport,
            configuration,
            connections: ConnectionList::new(max_connections),
            slots: ConnectionDataList::new(max_connections),
            address_to_id: HashMap::new(),
        }
    }

    /// Reserves a slot for a client to connect to.
    /// Clients can only connect to their reserved slots using the provided secret.
    /// 
    /// # Arguments
    /// 
    /// * `secret` - The client's connection secret.
    /// * `connection_token` - The client's publicly shared connection token.
    pub fn reserve(&mut self, secret: Secret, connection_token: ConnectionToken) -> Option<Connection> {
        let connection = self.connections.create_connection()?;

        debug_assert!(self.slots.get(connection).is_none(), "There is no data available for newly created connections");

        self.slots.set(connection, ConnectionSlot::Reserved(ReservedSlot {
            timeout: self.configuration.reserved_timeout,
            secret,
            connection_token,
        }));

        Some(connection)
    }

    pub fn update(&mut self) -> Vec<Event> {
        let mut poll_again = true; 
        let mut events = Vec::new();
        
        while poll_again {
            let mut buffer = [0; 1500];
            match self.transport.poll(&mut buffer) {
                Ok(result) => {
                    if let Some((length, address)) = result {
                        let packet = RawPacket::new(buffer, length);
                        
                        let connection = if let Some(connection) = self.address_to_id.get(&address) {
                            Some(*connection)
                        } else {
                            None
                        };

                        if let Some(connection) = connection {
                            self.handle_message(connection, packet, &mut events);
                        } else {
                            self.add_connection(address, packet, &mut events);
                        }
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
            if let Some(slot) = self.slots.get_mut(connection) {
                match slot {
                    ConnectionSlot::Empty => { debug_assert!(false, "This should never happen"); },
                    ConnectionSlot::Reserved(reserved_slot) => {
                        reserved_slot.timeout -= 1;
                        if reserved_slot.timeout == 0 {
                            self.connections.delete_connection(connection);
                            events.push(Event::Disconnected{ connection });
                        }
                    },
                    ConnectionSlot::Connected(connected_slot) => {
                        connected_slot.timeout -= 1;
                        if connected_slot.timeout == 0 {
                            self.connections.delete_connection(connection);
                            self.address_to_id.remove(&connected_slot.address);
                            events.push(Event::Disconnected{ connection });
                        }
                    },
                }
            } else {
                debug_assert!(false, "This should never happen");
            }
        }

        events
    }

    pub fn send(&mut self, buffer: OutgoingPacket, connection: Connection) -> Result<usize, TransportError> {

        if let Some(slot) = self.slots.get(connection) {
            match slot {
                ConnectionSlot::Connected(connected_slot) => {
                    let packet = buffer.write_header_and_sign(
                        0, 
                        0, 
                        [0x0, 0x0, 0x0, 0x0], 
                        0, 
                        &connected_slot.secret);

                    self.transport.send(&connected_slot.address, packet.get_buffer())
                },
                _ => {
                    panic!("TODO connection not connected");
                }
            }
        } else {
            panic!("TODO connection does not exist");
        }
    }

    fn add_connection(&mut self, address: SocketAddr, packet: RawPacket, events: &mut Vec<Event>) {

        // TODO parse connection packet, compare connection tokens, verify with secret

        if let Some(connection) = self.connections.create_connection() {
            self.address_to_id.insert(address, connection);
            self.slots.set(connection, ConnectionSlot::Connected(ConnectedSlot {
                address,
                secret: Secret::from_bytes([0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0]),
                sequence_number: 0,
                timeout: self.configuration.timeout,
                heartbeat: 0,
            }));

            events.push(Event::Connected {
                connection,
            });
        }
    }

    fn handle_message(&mut self, connection: Connection, packet: RawPacket, events: &mut Vec<Event>) {
        // TODO
    }
}

use crate::connection::*;
use super::transport::{Transport, TransportError};
use std::collections::HashMap;
use std::net::SocketAddr;
use crate::packets::{RawPacket, OutgoingPacket};
use crate::security::Secret;

mod configuration;
pub use configuration::Configuration;

mod event;
pub use event::Event;

/// The different states a connection slot can be in
#[derive(Debug, Eq, PartialEq)]
pub enum ConnectionState {
    /// The slot is unused and can be reserved.
    /// This is the default state for each connection.
    Empty,

    /// The slot is reserved and the client it is reserved for can connect to it.
    /// It will go back to being Empty if `reserved_timeout` ticks elapse without the client connecting succesfully.
    Reserved,

    /// A client is connected to this slot.
    /// It will go back to being Empty if `timeout` ticks elapse without receiving any data from the client.
    /// This is automatically mitigated by heartbeats and only occurs if the client can no longer reach the server.
    Connected,
}

pub struct Server {
    transport: Box<dyn Transport>,
    configuration: Configuration,
    connections: ConnectionList,
    connection_states: ConnectionDataList<ConnectionState>,
    address_to_id: HashMap<SocketAddr, Connection>,
    addresses: ConnectionDataList<SocketAddr>,
    timeouts: ConnectionDataList<usize>,
    sequence_numbers: ConnectionDataList<usize>,
    secrets: ConnectionDataList<Secret>,
}

impl Server {
    pub fn new(configuration: Configuration, transport: Box<dyn Transport>) -> Self {
        let max_connections = configuration.max_connections;
        Self {
            transport,
            configuration,
            connections: ConnectionList::new(max_connections),
            connection_states: ConnectionDataList::new(max_connections),
            address_to_id: HashMap::new(),
            addresses: ConnectionDataList::new(max_connections),
            timeouts: ConnectionDataList::new(max_connections),
            sequence_numbers: ConnectionDataList::new(max_connections),
            secrets: ConnectionDataList::new(max_connections),
        }
    }

    /// Reserves a slot for a client to connect to.
    /// Clients can only connect to their reserved slots using the provided secret.
    /// 
    /// # Arguments
    /// 
    /// * `secret` - The client's connection secret for the handshake.
    pub fn reserve(&mut self, secret: Secret) -> Option<Connection> {
        let connection = self.connections.create_connection()?;

        self.connection_states.set(connection, ConnectionState::Reserved);
        self.timeouts.set(connection, self.configuration.reserved_timeout);
        self.sequence_numbers.set(connection, 0);
        self.secrets.set(connection, secret);

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
            let timeout = self.timeouts.get(connection).unwrap().clone();

            if timeout == 0 {
                //connection timeout
                let address = self.addresses.get(connection).unwrap();
                self.address_to_id.remove(address);
                self.connections.delete_connection(connection).unwrap();
                events.push(Event::Disconnected{ connection });
            } else {
                self.timeouts.set(connection, timeout - 1);
            }
        }

        events
    }

    pub fn send(&mut self, buffer: OutgoingPacket, connection: Connection) -> Result<usize, TransportError> {
        let address = self.addresses.get(connection).expect("TODO");
        let secret = self.secrets.get(connection).unwrap();

        let packet = buffer.write_header_and_sign(0, 0, [0x0, 0x0, 0x0, 0x0], 0, secret);

        self.transport.send(address, packet.get_buffer())
    }

    fn add_connection(&mut self, address: SocketAddr, packet: RawPacket, events: &mut Vec<Event>) {
        if let Some(connection) = self.connections.create_connection() {
            self.address_to_id.insert(address, connection);
            self.timeouts.set(connection, self.configuration.timeout);
            self.addresses.set(connection, address);
            
            events.push(Event::Connected {
                connection: connection.clone(),
            });
        }
    }

    fn handle_message(&mut self, connection: Connection, packet: RawPacket, events: &mut Vec<Event>) {
        let secret = self.secrets.get(connection).unwrap();

        if let Some(incoming) = packet.verify(secret) {
            self.timeouts.set(connection, self.configuration.timeout);
            
            events.push(Event::Message {
                connection: connection.clone(),
                payload: incoming.into_payload(),
            });
        } else {
            println!("got invalid packet");
        }
    }
}

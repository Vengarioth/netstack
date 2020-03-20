use crate::connection::*;
use super::transport::{Transport, TransportError};
use std::net::SocketAddr;
use std::collections::HashMap;
use crate::packets::RawPacket;

mod configuration;
pub use configuration::Configuration;

mod event;
pub use event::Event;

use crate::security::Secret;
use crate::packets::OutgoingPacket;

#[derive(Debug, Eq, PartialEq)]
pub enum ConnectionState {
    Disconnected,
    Connecting,
    Connected,
}

pub struct Client {
    configuration: Configuration,
    transport: Box<dyn Transport>,
    connections: ConnectionList,
    states: ConnectionDataList<ConnectionState>,
    addresses: ConnectionDataList<SocketAddr>,
    address_to_connection: HashMap<SocketAddr, Connection>,
    timeouts: ConnectionDataList<usize>,
    secrets: ConnectionDataList<Secret>,
}

impl Client {
    pub fn new(configuration: Configuration, transport: Box<dyn Transport>) -> Self {
        let max_connections = configuration.max_connections;

        Self {
            configuration,
            transport,
            connections: ConnectionList::new(max_connections),
            states: ConnectionDataList::new(max_connections),
            addresses: ConnectionDataList::new(max_connections),
            address_to_connection: HashMap::new(),
            timeouts: ConnectionDataList::new(max_connections),
            secrets: ConnectionDataList::new(max_connections),
        }
    }

    pub fn connect(&mut self, remote_address: SocketAddr, secret: Secret) -> Result<Connection, ()> {

        if self.address_to_connection.contains_key(&remote_address) {
            return Err(());
        }

        let connection = self.connections.create_connection().expect("TODO");
        self.addresses.set(connection, remote_address);
        self.address_to_connection.insert(remote_address, connection);
        self.timeouts.set(connection, self.configuration.timeout);
        self.states.set(connection, ConnectionState::Connecting);
        self.secrets.set(connection, secret);

        Ok(connection)
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

                        let connection = if let Some(c) = self.address_to_connection.get(&address) {
                            Some(*c)
                        } else {
                            None
                        };

                        if let Some(connection) = connection {
                            self.handle_message(connection, packet, &mut events);
                        } else {
                            println!("packet from unknown source");
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
                self.address_to_connection.remove(address);
                self.connections.delete_connection(connection).unwrap();
                self.states.set(connection, ConnectionState::Disconnected);
                events.push(Event::Disconnected { connection });
            } else {
                self.timeouts.set(connection, timeout - 1);
            }
        }

        events
    }

    pub fn send(&mut self, packet: OutgoingPacket, connection: Connection) -> Result<usize, TransportError> {
        let state = self.states.get(connection).unwrap();
        if state == &ConnectionState::Disconnected {
            panic!("not connected");
        }

        // TODO
        let secret = self.secrets.get(connection).unwrap();
        
        let raw = packet.write_header_and_sign(0, 0, [0x0, 0x0, 0x0, 0x0], 0, secret);

        let address = self.addresses.get(connection).unwrap();
        println!("sending message to {}", address);
        self.transport.send(address, raw.get_buffer())
    }

    fn handle_message(&mut self, connection: Connection, packet: RawPacket, events: &mut Vec<Event>) {

        let secret = self.secrets.get(connection).unwrap();
        if let Some(incoming) = packet.verify(secret) {

            let state = self.states.get(connection).unwrap();
            if *state == ConnectionState::Connecting {
                self.states.set(connection, ConnectionState::Connected);
                events.push(Event::Connected { connection });
            }

            self.timeouts.set(connection, self.configuration.timeout);

            events.push(Event::Message {
                connection,
                payload: incoming.into_payload(),
            });

        } else {
            println!("got invalid packet");
        }
    }
}

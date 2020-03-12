use crate::connection::*;
use super::transport::{Transport, TransportError};
use std::net::SocketAddr;
use std::collections::HashMap;

mod configuration;
pub use configuration::Configuration;

mod event;
pub use event::Event;

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
        }
    }

    pub fn connect(&mut self, remote_address: SocketAddr) -> Result<Connection, ()> {

        if self.address_to_connection.contains_key(&remote_address) {
            return Err(());
        }

        let connection = self.connections.create_connection().expect("TODO");
        self.addresses.set(connection, remote_address);
        self.address_to_connection.insert(remote_address, connection);
        self.timeouts.set(connection, self.configuration.timeout);
        self.states.set(connection, ConnectionState::Connecting);

        Ok(connection)
    }

    pub fn update(&mut self) -> Vec<Event> {
        let mut poll_again = true;
        let mut events = Vec::new();
        let mut buffer: [u8; 1500] = [0; 1500];

        while poll_again {
            match self.transport.poll(&mut buffer) {
                Ok(result) => {
                    if let Some((length, address)) = result {
                        if let Some(connection) = self.address_to_connection.get(&address) {
                            let connection = connection.clone();
                            let state = self.states.get(connection).unwrap();
                            if *state == ConnectionState::Connecting {
                                self.states.set(connection, ConnectionState::Connected);
                                events.push(Event::Connected { connection });
                            }

                            events.push(Event::Message {
                                buffer,
                                length,
                                connection,
                            });
                            buffer = [0; 1500];
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

    pub fn send(&mut self, buffer: &[u8], connection: Connection) -> Result<usize, TransportError> {
        let state = self.states.get(connection).unwrap();
        if state == &ConnectionState::Disconnected {
            panic!("not connected");
        }

        let address = self.addresses.get(connection).unwrap();
        println!("sending message to {}", address);
        self.transport.send(address, buffer)
    }
}

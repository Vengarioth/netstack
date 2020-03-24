use failure::Error;
use crate::connection::*;
use super::transport::Transport;
use std::net::SocketAddr;
use std::collections::HashMap;
use crate::packets::RawPacket;

mod configuration;
pub use configuration::Configuration;

mod event;
pub use event::Event;

mod error;
pub use error::ClientError;

use crate::security::{Secret, ConnectionToken, ReplayBuffer};
use crate::packets::{OutgoingPacket, PacketType};

#[derive(Debug, Eq, PartialEq, Copy, Clone)]
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
    timeouts: ConnectionDataList<usize>,
    heartbeats: ConnectionDataList<usize>,
    sequence_numbers: ConnectionDataList<u64>,
    secrets: ConnectionDataList<Secret>,
    replay_buffers: ConnectionDataList<ReplayBuffer>,
    ack_buffers: ConnectionDataList<ReplayBuffer>,
    connection_tokens: ConnectionDataList<ConnectionToken>,
    address_to_connection: HashMap<SocketAddr, Connection>,
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
            timeouts: ConnectionDataList::new(max_connections),
            heartbeats: ConnectionDataList::new(max_connections),
            sequence_numbers: ConnectionDataList::new(max_connections),
            secrets: ConnectionDataList::new(max_connections),
            replay_buffers: ConnectionDataList::new(max_connections),
            ack_buffers: ConnectionDataList::new(max_connections),
            connection_tokens: ConnectionDataList::new(max_connections),
            address_to_connection: HashMap::new(),
        }
    }

    pub fn connect(&mut self, remote_address: SocketAddr, secret: Secret, connection_token: ConnectionToken) -> Result<Connection, Error> {

        if self.address_to_connection.contains_key(&remote_address) {
            return Err(ClientError::AlreadyConnectedToAddress{ address: remote_address }.into());
        }

        if let Some(connection) = self.connections.create_connection() {

            self.addresses.set(connection, remote_address);
            self.address_to_connection.insert(remote_address, connection);
            self.timeouts.set(connection, self.configuration.timeout);
            self.heartbeats.set(connection, self.configuration.heartbeat);
            self.sequence_numbers.set(connection, 0);
            self.states.set(connection, ConnectionState::Connecting);
            self.secrets.set(connection, secret);
            self.replay_buffers.set(connection, ReplayBuffer::new());
            self.ack_buffers.set(connection, ReplayBuffer::new());
            self.connection_tokens.set(connection, connection_token);

            self.send_connection_message(connection)?;

            Ok(connection)

        } else {
            Err(ClientError::MaximumConnectionsReached.into())
        }
    }

    pub fn update(&mut self) -> Vec<Event> {
        let mut poll_again = true;
        let mut events = Vec::new();
        
        while poll_again {
            let mut buffer = [0; 1500];
            match self.transport.poll(&mut buffer) {
                Ok(Some((length, address))) => {
                    let packet = RawPacket::new(buffer, length);

                    if let Some(connection) = self.find_connection(&address) {
                        self.handle_message(connection, packet, &mut events);
                    } else {
                        println!("message from unknown source");
                    }
                },
                Ok(None) => {
                    poll_again = false;
                },
                Err(error) => {
                    println!("{}", error);
                },
            }
        }

        let connections: Vec<Connection> = self.connections.into_iter().collect();
        for connection in connections {
            
            // manage timeouts
            let timeout = self.timeouts.get(connection).expect("No timeout set for connection") - 1;
            if timeout == 0 {
                let address = self.addresses.get(connection).unwrap();
                
                self.address_to_connection.remove(address);
                self.states.set(connection, ConnectionState::Disconnected);
                self.addresses.remove(connection);
                self.timeouts.remove(connection);
                self.heartbeats.remove(connection);
                self.sequence_numbers.remove(connection);
                self.secrets.remove(connection);
                self.connection_tokens.remove(connection);

                self.connections.delete_connection(connection).unwrap();
                events.push(Event::Disconnected { connection });
                continue;

            } else {
                self.timeouts.set(connection, timeout);
            }

            // manage heartbeats
            let heartbeat = self.heartbeats.get(connection).expect("No heartbeat set for connection") - 1;
            if heartbeat == 0 {

                let state = self.get_connection_state(connection);
                match state {
                    Some(ConnectionState::Connected) => {
                        self.send_heartbeat_message(connection).expect("Could not send heartbeat message");
                    },
                    Some(ConnectionState::Connecting) => {
                        self.send_connection_message(connection).expect("Could not send connection message");
                    },
                    _ => {
                        panic!("this should not happen");
                    }
                }
            } else {
                self.heartbeats.set(connection, heartbeat);
            }
        }

        events
    }

    /// Sends a packet to the given connection and returns the packet's sequence number
    pub fn send(&mut self, packet: OutgoingPacket, connection: Connection) -> Result<u64, Error> {
        match self.get_connection_state(connection) {
            Some(ConnectionState::Connected) => {
                Ok(self.send_internal(packet, connection, PacketType::Payload)?)
            },
            Some(ConnectionState::Connecting) => {
                Err(ClientError::ConnectionStillConnecting.into())
            },
            Some(ConnectionState::Disconnected) => {
                Err(ClientError::ConnectionDisconnected.into())
            },
            None => {
                Err(ClientError::ConnectionNotFound.into())
            },
        }
    }

    fn send_internal(&mut self, packet: OutgoingPacket, connection: Connection, packet_type: PacketType) -> Result<u64, Error> {
        let sequence_number = self.sequence_numbers.get(connection).expect("No sequence number for connection found") + 1;
        self.sequence_numbers.set(connection, sequence_number);
        let secret = self.secrets.get(connection).expect("No secret for connection found");

        let (ack_sequence_number, ack_bits) = self.replay_buffers.get(connection).expect("no replay buffer for connection").get_ack_bits();

        let raw = packet.write_header_and_sign(sequence_number, ack_sequence_number, ack_bits, packet_type.to_u8(), secret);
        let address = self.addresses.get(connection).expect("No address for connection found");

        // TODO check bytes sent?
        let _bytes_sent = self.transport.send(address, raw.get_buffer())?;

        self.heartbeats.set(connection, self.configuration.heartbeat);

        Ok(sequence_number)
    }

    fn find_connection(&self, address: &SocketAddr) -> Option<Connection> {
        match self.address_to_connection.get(&address) {
            Some(c) => Some(*c),
            None => None,
        }
    }

    fn get_connection_state(&self, connection: Connection) -> Option<ConnectionState> {
        match self.states.get(connection) {
            Some(c) => Some(*c),
            None => None,
        }
    }

    fn handle_message(&mut self, connection: Connection, packet: RawPacket, events: &mut Vec<Event>) {
        let secret = self.secrets.get(connection).expect("Secret for connection not found");

        if let Some(incoming) = packet.verify(secret) {
            let state = self.states.get(connection).expect("State for connection not found").clone();

            let sequence_number = incoming.get_sequence_number();
            let replay_buffer = self.replay_buffers.get_mut(connection).expect("No replay buffer for connection");

            if replay_buffer.acknowledge(sequence_number) {
                let ack_sequence_number = incoming.get_ack_sequence_number();
                let ack_bits = incoming.get_ack_bits();

                let ack_buffer = self.ack_buffers.get_mut(connection).expect("no replay buffer for connection");
                let acked = ack_buffer.set_ack_bits(ack_sequence_number, ack_bits);

                for sequence_number in acked {
                    events.push(Event::MessageAcknowledged {
                        connection,
                        sequence_number,
                    });
                }

                if state == ConnectionState::Connecting {
                    self.states.set(connection, ConnectionState::Connected);
                    self.timeouts.set(connection, self.configuration.timeout);
                    self.heartbeats.set(connection, self.configuration.heartbeat);
                    self.connection_tokens.remove(connection);

                    events.push(Event::Connected { connection });

                    if incoming.get_packet_type() == Some(PacketType::Payload) {
                        events.push(Event::Message {
                            connection,
                            payload: incoming.into_payload(),
                        });
                    }
                } else {

                    match incoming.get_packet_type() {
                        Some(PacketType::Payload) => {
                            self.timeouts.set(connection, self.configuration.timeout);

                            events.push(Event::Message {
                                connection,
                                payload: incoming.into_payload(),
                            });
                        },
                        Some(PacketType::Heartbeat) => {
                            self.timeouts.set(connection, self.configuration.timeout);
                        },
                        Some(packet_type) => {
                            println!("got unexpected packet type {:?}", packet_type);
                        }
                        _ => {
                            println!("got invalid packet type");
                        }
                    }
                }
            } else {
                println!("got packet with unusable sequence number");
            }
        } else {
            println!("got invalid packet");
        }
    }

    fn send_connection_message(&mut self, connection: Connection) -> Result<(), Error> {
        use std::io::Write;
        
        let connection_token = self.connection_tokens.get(connection).expect("No connection token for connection");

        let mut packet = OutgoingPacket::new();
        packet.write(connection_token.get_bytes())?;
        
        self.send_internal(packet, connection, PacketType::Connection)?;

        Ok(())
    }

    fn send_heartbeat_message(&mut self, connection: Connection) -> Result<(), Error> {
        let packet = OutgoingPacket::new();
        self.send_internal(packet, connection, PacketType::Heartbeat)?;

        Ok(())
    }
}

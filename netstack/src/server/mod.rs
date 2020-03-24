use failure::Error;
use crate::connection::*;
use super::transport::Transport;
use std::collections::HashMap;
use std::net::SocketAddr;
use crate::packets::{RawPacket, OutgoingPacket, PacketType};
use crate::security::{Secret, ConnectionToken, ReplayBuffer};

mod configuration;
pub use configuration::Configuration;

mod event;
pub use event::Event;

mod error;
pub use error::ServerError;

#[derive(Debug, Eq, PartialEq, Copy, Clone)]
pub enum ConnectionState {
    Empty,
    Reserved,
    Connected,
}

pub struct Server {
    transport: Box<dyn Transport>,
    configuration: Configuration,
    connections: ConnectionList,
    states: ConnectionDataList<ConnectionState>,
    addresses: ConnectionDataList<SocketAddr>,
    timeouts: ConnectionDataList<usize>,
    heartbeats: ConnectionDataList<usize>,
    sequence_numbers: ConnectionDataList<u64>,
    secrets: ConnectionDataList<Secret>,
    replay_buffers: ConnectionDataList<ReplayBuffer>,
    ack_buffers: ConnectionDataList<ReplayBuffer>,
    connection_token_to_connection: HashMap<ConnectionToken, Connection>,
    address_to_connection: HashMap<SocketAddr, Connection>,
}

impl Server {
    pub fn new(configuration: Configuration, transport: Box<dyn Transport>) -> Self {
        let max_connections = configuration.max_connections;
        Self {
            transport,
            configuration,
            connections: ConnectionList::new(max_connections),
            states: ConnectionDataList::new(max_connections),
            addresses: ConnectionDataList::new(max_connections),
            timeouts: ConnectionDataList::new(max_connections),
            heartbeats: ConnectionDataList::new(max_connections),
            sequence_numbers: ConnectionDataList::new(max_connections),
            secrets: ConnectionDataList::new(max_connections),
            replay_buffers: ConnectionDataList::new(max_connections),
            ack_buffers: ConnectionDataList::new(max_connections),
            connection_token_to_connection: HashMap::new(),
            address_to_connection: HashMap::new(),
        }
    }

    /// Reserves a slot for a client to connect to.
    /// Clients can only connect to their reserved slots using the provided secret.
    /// 
    /// # Arguments
    /// 
    /// * `secret` - The client's connection secret.
    /// * `connection_token` - The client's publicly shared connection token.
    pub fn reserve(&mut self, secret: Secret, connection_token: ConnectionToken) -> Result<Connection, Error> {
        
        if let Some(connection) = self.connections.create_connection() {

            self.states.set(connection, ConnectionState::Reserved);
            self.secrets.set(connection, secret);
            self.timeouts.set(connection, self.configuration.reserved_timeout);
            self.replay_buffers.set(connection, ReplayBuffer::new());
            self.ack_buffers.set(connection, ReplayBuffer::new());
            self.connection_token_to_connection.insert(connection_token, connection);

            Ok(connection)

        } else {
            Err(ServerError::MaximumConnectionsReached.into())
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
                        self.add_connection(address, packet, &mut events);
                    }
                },
                Ok(None) => {
                    poll_again = false;
                },
                Err(error) => {
                    println!("TransportError: {}", error);
                },
            }
        }

        let connections: Vec<Connection> = self.connections.into_iter().collect();
        for connection in connections {

            let timeout = self.timeouts.get(connection).expect("No timeout set for connection") - 1;
            if timeout == 0 {
                let address = self.addresses.get(connection).unwrap();
                
                self.address_to_connection.remove(address);
                self.states.set(connection, ConnectionState::Empty);
                self.addresses.remove(connection);
                self.timeouts.remove(connection);
                self.heartbeats.remove(connection);
                self.sequence_numbers.remove(connection);
                self.secrets.remove(connection);

                self.connections.delete_connection(connection).unwrap();
                events.push(Event::Disconnected { connection });
                continue;
            } else {
                self.timeouts.set(connection, timeout);
            }

            let state = self.states.get(connection).expect("No state set for connection").clone();

            if state == ConnectionState::Connected {
                let heartbeat = self.heartbeats.get(connection).expect("No heartbeat set for connection") - 1;
                if heartbeat == 0 {
                    self.send_heartbeat_message(connection).expect("Could not send heartbeat message");
                } else {
                    self.heartbeats.set(connection, heartbeat);
                }
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
            Some(ConnectionState::Reserved) => {
                Err(ServerError::ConnectionNotReady.into())
            },
            _ => {
                Err(ServerError::ConnectionNotFound.into())
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

    fn add_connection(&mut self, address: SocketAddr, packet: RawPacket, events: &mut Vec<Event>) {

        // --- careful about mutating state before the packed is valid ---

        let packet_type = if let Some(packet_type) = PacketType::from_u8(packet.get_header().packet_type) {
            packet_type
        } else {
            println!("got unknown packet type");
            return;
        };

        if packet_type != PacketType::Connection {
            println!("got invalid packet type {:?}, expected {:?}", packet_type, PacketType::Connection);
            return;
        }

        let connection_token = if let Ok(connection_token) = ConnectionToken::from_slice(packet.get_body()) {
            connection_token
        } else {
            println!("could not get connection token from connection message");
            return;
        };
        
        let connection = if let Some(connection) = self.connection_token_to_connection.get(&connection_token) {
            connection.clone()
        } else {
            println!("no connection found for connection token");
            return;
        };

        let secret = self.secrets.get(connection).expect("no secret found for connection");
        let packet = if let Some(packet) = packet.verify(&secret) {
            packet
        } else {
            println!("connection packet was invalid");
            return;
        };

        // --- here the packet is validated and we can begin to change state based on it ---

        let replay_buffer = self.replay_buffers.get_mut(connection).expect("no replay buffer for connection");
        if !replay_buffer.acknowledge(packet.get_sequence_number()) {
            println!("got packet with invalid sequence number");
            return;
        }

        let ack_sequence_number = packet.get_ack_sequence_number();
        let ack_bits = packet.get_ack_bits();

        let ack_buffer = self.ack_buffers.get_mut(connection).expect("no replay buffer for connection");
        let acked = ack_buffer.set_ack_bits(ack_sequence_number, ack_bits);

        for sequence_number in acked {
            events.push(Event::MessageAcknowledged {
                connection,
                sequence_number,
            });
        }

        self.connection_token_to_connection.remove(&connection_token);

        self.states.set(connection, ConnectionState::Connected);
        self.addresses.set(connection, address);
        self.timeouts.set(connection, self.configuration.timeout);
        self.heartbeats.set(connection, self.configuration.heartbeat);
        self.sequence_numbers.set(connection, 0);
        self.address_to_connection.insert(address, connection);

        events.push(Event::Connected {
            connection,
        });
    }

    fn handle_message(&mut self, connection: Connection, packet: RawPacket, events: &mut Vec<Event>) {
        let secret = self.secrets.get(connection).expect("No secret for connection");

        if let Some(packet) = packet.verify(secret) {

            let sequence_number = packet.get_sequence_number();
            let replay_buffer = self.replay_buffers.get_mut(connection).expect("no replay buffer for connection");

            if replay_buffer.acknowledge(sequence_number) {

                let ack_sequence_number = packet.get_ack_sequence_number();
                let ack_bits = packet.get_ack_bits();

                let ack_buffer = self.ack_buffers.get_mut(connection).expect("no replay buffer for connection");
                let acked = ack_buffer.set_ack_bits(ack_sequence_number, ack_bits);

                for sequence_number in acked {
                    events.push(Event::MessageAcknowledged {
                        connection,
                        sequence_number,
                    });
                }

                match packet.get_packet_type() {
                    Some(PacketType::Payload) => {
                        self.timeouts.set(connection, self.configuration.timeout);

                        events.push(Event::Message {
                            connection,
                            payload: packet.into_payload(),
                        });
                    },
                    Some(PacketType::Heartbeat) => {
                        self.timeouts.set(connection, self.configuration.timeout);
                    },
                    Some(packet_type) => {
                        println!("unexpected packet type {:?}", packet_type);
                    },
                    None => {
                        println!("invalid packet type");
                    }
                }
            } else {
                println!("unusable sequence number");
            }

        } else {
            println!("got invalid packet");
        }
    }

    fn send_heartbeat_message(&mut self, connection: Connection) -> Result<(), Error> {
        let packet = OutgoingPacket::new();
        self.send_internal(packet, connection, PacketType::Heartbeat)?;

        Ok(())
    }
}

use netstack::{
    client::{
        Client,
        Configuration,
        Event,
    },
    transport::UdpTransport,
    time::Clock,
    security::{
        Secret,
        ConnectionToken
    },
    packets::OutgoingPacket,
    monitoring::EmptyClientMonitor,
};
use std::net::SocketAddr;
use std::time::Duration;
use std::io::Write;
use serde::{Deserialize, Serialize};
use base58::FromBase58;

#[derive(Serialize, Deserialize)]
pub struct ConnectionInfo {
    token: String,
    secret: String,
}

fn get_connection_info() -> ConnectionInfo {
    let response = ureq::get("http://127.0.0.1:8000/token").call();

    if !response.ok() {
        panic!("could not get a secret from the remote server");
    }

    let data = response.into_string().unwrap();

    serde_json::from_str(&data).unwrap()
}

fn main() {
    let mut clock = Clock::new(Duration::from_millis(100));

    let local_address: SocketAddr = "127.0.0.1:0".parse().unwrap();
    let remote_address: SocketAddr = "127.0.0.1:9000".parse().unwrap();
    let transport = UdpTransport::new(local_address).unwrap();

    let config = Configuration {
        max_connections: 6,
        timeout: 120,
        heartbeat: 60,
    };

    let monitor = EmptyClientMonitor::new();

    let mut client = Client::new(config, Box::new(transport), Box::new(monitor));

    let connection_info = get_connection_info();

    let secret = Secret::from_slice(&connection_info.secret.from_base58().unwrap()).unwrap();
    let connection_token = ConnectionToken::from_slice(&connection_info.token.from_base58().unwrap()).unwrap();

    let server = client.connect(remote_address, secret, connection_token).unwrap();

    let mut connected = false;
    loop {
        if clock.update() {
            let events = client.update();

            for event in events {
                match event {
                    Event::Connected { .. } => {
                        connected = true;
                        println!("connected to a server");
                    },
                    Event::Disconnected { .. } => {
                        connected = false;
                        println!("disconnected from a server");
                    },
                    Event::Message { .. } => {
                        println!("got a message from a server");
                    },
                    Event::MessageAcknowledged{ connection, sequence_number } => {
                        println!("Message {} sent to {} got acknowledged", sequence_number, connection);
                    },
                }
            }
            
            if connected {
                let mut packet = OutgoingPacket::new();
                packet.write(&[0x1, 0x2, 0x3, 0x4]).unwrap();
                
                let sequence_number = client.send(packet, server).unwrap();
                println!("Sent Message {} to server", sequence_number);
            }
        }
    }
}

use netstack::{
    client::{
        Client,
        Configuration,
        Event,
    },
    transport::UdpTransport,
    time::Clock,
    security::Secret,
    packets::OutgoingPacket,
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
    let response = ureq::get("http://127.0.0.1:8000/").call();

    if !response.ok() {
        panic!("could not get a secret from the remote server");
    }

    let data = response.into_string().unwrap();

    serde_json::from_str(&data).unwrap()
}

fn main() {
    let mut clock = Clock::new(Duration::from_millis(16));

    let local_address: SocketAddr = "127.0.0.1:0".parse().unwrap();
    let remote_address: SocketAddr = "127.0.0.1:9000".parse().unwrap();
    let transport = UdpTransport::new(local_address).unwrap();

    let config = Configuration {
        max_connections: 6,
        timeout: 60
    };

    let mut client = Client::new(config, Box::new(transport));

    let connection_info = get_connection_info();

    let secret = Secret::from_slice(&connection_info.secret.from_base58().unwrap()).unwrap();

    let server = client.connect(remote_address, secret).unwrap();

    loop {
        if clock.update() {
            let events = client.update();

            for event in events {
                match event {
                    Event::Connected { .. } => {
                        println!("connected");
                    },
                    Event::Disconnected { .. } => {
                        println!("disconnected");
                    },
                    Event::Message { .. } => {
                        println!("message");
                    }
                }
            }

            let mut packet = OutgoingPacket::new();
            packet.write(&[0x1, 0x2, 0x3, 0x4]).unwrap();

            client.send(packet, server).unwrap();
        }
    }
}

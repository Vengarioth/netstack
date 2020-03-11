#[macro_use]
extern crate netstack_derive;

use std::net::SocketAddr;
use std::time::Duration;
use netstack::{
    server::{
        Configuration,
        Server,
        Event,
    },
    transport::UdpTransport,
    time::Clock,
};

fn main() {
    let local_address: SocketAddr = "127.0.0.1:9000".parse().unwrap();
    let transport = UdpTransport::new(local_address).unwrap();
    let mut clock = Clock::new(Duration::from_millis(16));

    let config = Configuration {
        max_connections: 64,
        timeout: 60,
    };

    let mut server = Server::new(config, Box::new(transport));

    loop {
        if clock.update() {
            let events = server.update();
    
            for event in events {
                match event {
                    Event::Connected { connection } => {
                        println!("{} connected", connection);
                    },
                    Event::Disconnected { connection } => {
                        println!("{} disconnected", connection);
                    },
                    Event::Message{ connection, length, .. } => {
                        println!("Message({}) from {}", length, connection);
                    }
                }
            }
        }
    }
}

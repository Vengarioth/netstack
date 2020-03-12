#[macro_use]
extern crate netstack_derive;

use netstack::{
    client::{
        Client,
        Configuration,
        Event,
    },
    transport::UdpTransport,
    time::Clock,
};
use std::net::SocketAddr;
use std::time::Duration;

#[derive(Debug, Serialize, Deserialize)]
pub struct Greeting {
    pub id: u32,
    pub to: String,
    pub message: String,
}

fn main() {
    let mut clock = Clock::new(Duration::from_millis(16));

    let greeting = Greeting {
        id: 42,
        to: "world".to_owned(),
        message: "hello world!".to_owned(),
    };

    dbg!(greeting);

    let local_address: SocketAddr = "127.0.0.1:0".parse().unwrap();
    let remote_address: SocketAddr = "127.0.0.1:9000".parse().unwrap();
    let transport = UdpTransport::new(local_address).unwrap();

    let config = Configuration {
        max_connections: 6,
        timeout: 60
    };

    let mut client = Client::new(config, Box::new(transport));

    let server = client.connect(remote_address).unwrap();

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

            client.send(&[0x1, 0x2, 0x3, 0x4], server).unwrap();
        }
    }
}

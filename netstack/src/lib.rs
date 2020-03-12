#[macro_use]
extern crate failure;

pub type Buffer = [u8; 1500];

pub mod serialization;
pub mod server;
pub mod client;
pub mod transport;
pub mod time;
pub mod connection;

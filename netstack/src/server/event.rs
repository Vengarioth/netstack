use crate::{
    connection::Connection,
    packets::Payload,
};

pub enum Event {
    Connected {
        connection: Connection,
    },
    Disconnected {
        connection: Connection,
    },
    Message{
        connection: Connection,
        payload: Payload,
    }
}

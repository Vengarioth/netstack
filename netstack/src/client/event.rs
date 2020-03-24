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
    Message {
        connection: Connection,
        payload: Payload,
    },
    MessageAcknowledged {
        connection: Connection,
        sequence_number: u64,
    },
}

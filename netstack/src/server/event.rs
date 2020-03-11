use super::Connection;

pub enum Event {
    Connected {
        connection: Connection,
    },
    Disconnected {
        connection: Connection,
    },
    Message{
        connection: Connection,
        buffer: [u8; 1500],
        length: usize,
    }
}

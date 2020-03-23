use std::net::SocketAddr;

#[derive(Debug, Fail)]
pub enum ClientError {
    #[fail(display = "Already connected to socket address {}", address)]
    AlreadyConnectedToAddress { address: SocketAddr },

    #[fail(display = "Maximum number of connections reached")]
    MaximumConnectionsReached,

    #[fail(display = "Connection still connecting, please wait for the connected event before sending packets")]
    ConnectionStillConnecting,

    #[fail(display = "Connection disconnected")]
    ConnectionDisconnected,

    #[fail(display = "Connection not found")]
    ConnectionNotFound,
}

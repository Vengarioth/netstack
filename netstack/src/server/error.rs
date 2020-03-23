#[derive(Debug, Fail)]
pub enum ServerError {
    #[fail(display = "Maximum number of connections reached")]
    MaximumConnectionsReached,

    #[fail(display = "Connection not ready, please wait for the connected event before sending packets")]
    ConnectionNotReady,

    #[fail(display = "Connection not found")]
    ConnectionNotFound,
}

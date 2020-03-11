use std::io::Error as IOError;

#[derive(Debug, Fail)]
pub enum TransportError {
    #[fail(display = "An IO error occured: {}", inner)]
    IOError {
        inner: IOError,
    }
}

impl From<IOError> for TransportError {
    fn from(inner: IOError) -> Self {
        Self::IOError {
            inner,
        }
    }
}

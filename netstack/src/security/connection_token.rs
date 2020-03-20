const CONNECTION_TOKEN_SIZE: usize = 32;

#[derive(Debug, Eq, PartialEq)]
pub struct ConnectionToken([u8; CONNECTION_TOKEN_SIZE]);

impl ConnectionToken {
    pub fn from_bytes(bytes: [u8; CONNECTION_TOKEN_SIZE]) -> Self {
        Self(bytes)
    }

    pub fn get_bytes(&self) -> &[u8] {
        &self.0
    }
}

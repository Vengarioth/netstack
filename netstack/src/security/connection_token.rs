const CONNECTION_TOKEN_SIZE: usize = 32;

#[derive(Debug, Eq, PartialEq, Clone)]
pub struct ConnectionToken([u8; CONNECTION_TOKEN_SIZE]);

impl ConnectionToken {
    pub fn from_slice(slice: &[u8]) -> Result<Self, ()> {
        if slice.len() != CONNECTION_TOKEN_SIZE {
            panic!("TODO slice wrong size")
        }

        let mut bytes = [0; CONNECTION_TOKEN_SIZE];
        for i in 0..CONNECTION_TOKEN_SIZE {
            bytes[i] = slice[i];
        }

        Ok(Self(bytes))
    }

    pub fn from_bytes(bytes: [u8; CONNECTION_TOKEN_SIZE]) -> Self {
        Self(bytes)
    }

    pub fn get_bytes(&self) -> &[u8] {
        &self.0
    }
}

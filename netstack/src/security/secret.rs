const SECRET_SIZE: usize = 32;

#[derive(Debug, Eq, PartialEq, Clone)]
pub struct Secret([u8; SECRET_SIZE]);

impl Secret {
    pub fn from_slice(slice: &[u8]) -> Result<Self, ()> {
        if slice.len() != SECRET_SIZE {
            panic!("TODO slice wrong size")
        }

        let mut bytes = [0; SECRET_SIZE];
        for i in 0..SECRET_SIZE {
            bytes[i] = slice[i];
        }

        Ok(Self(bytes))
    }
    
    pub fn from_bytes(bytes: [u8; SECRET_SIZE]) -> Self {
        Self(bytes)
    }

    pub fn get_bytes(&self) -> &[u8] {
        &self.0
    }
}

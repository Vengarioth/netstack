use super::{Deserializer, SerializationError};
use super::zig_zag::decode;

const VARINT_MAX_ITERATIONS: usize = 10;

pub struct Reader<'a> {
    buffer: &'a [u8],
    position: usize,
}

impl<'a> Reader<'a> {
    pub fn new(buffer: &'a [u8]) -> Self {
        Self {
            buffer,
            position: 0,
        }
    }

    fn read_var_uint(&mut self) -> Result<u64, SerializationError> {
        let start = self.position;
        let mut bytes_read = 0;

        let mut decoded = 0;
        let mut shift = 0;

        for i in 0..VARINT_MAX_ITERATIONS {
            let value = self.buffer[start + i];
            bytes_read += 1;

            decoded |= ((value & 0x7f) as u64) << shift;

            if (value & 0x80) != 0x80 {
                self.position += bytes_read;
                return Ok(decoded);
            }

            shift += 7;
        }

        Err(SerializationError::VarUIntOverflow)
    }

    fn read_var_int(&mut self) -> Result<i64, SerializationError> {
        let value = self.read_var_uint()?;
        Ok(decode(value))
    }
}

impl<'a> Deserializer for Reader<'a> {
    fn deserialize_string(&mut self) -> Result<String, SerializationError> {
        let length = self.read_var_uint()? as usize;
        let start = self.position;
        let end = self.position + length;

        self.position += length;

        Ok(std::str::from_utf8(&self.buffer[start..end]).unwrap().to_owned())
    }

    fn deserialize_u8(&mut self) -> Result<u8, SerializationError> {
        let position = self.position;
        self.position += 1;
        Ok(self.buffer[position])
    }

    fn deserialize_i8(&mut self) -> Result<i8, SerializationError> {
        let position = self.position;
        self.position += 1;

        Ok(self.buffer[position] as i8)
    }
    
    fn deserialize_u16(&mut self) -> Result<u16, SerializationError> { unimplemented!() }
    fn deserialize_i16(&mut self) -> Result<i16, SerializationError> { unimplemented!() }
    fn deserialize_u32(&mut self) -> Result<u32, SerializationError> { unimplemented!() }
    fn deserialize_i32(&mut self) -> Result<i32, SerializationError> { unimplemented!() }
    fn deserialize_u64(&mut self) -> Result<u64, SerializationError> { unimplemented!() }
    fn deserialize_i64(&mut self) -> Result<i64, SerializationError> { unimplemented!() }
    fn deserialize_f32(&mut self) -> Result<f32, SerializationError> { unimplemented!() }
    fn deserialize_f64(&mut self) -> Result<f64, SerializationError> { unimplemented!() }
}

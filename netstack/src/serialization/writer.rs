use super::{Serializer, SerializationError};

pub struct Writer<'a> {
    buffer: &'a mut [u8],
    position: usize,
}

impl<'a> Writer<'a> {
    pub fn new(buffer: &'a mut [u8]) -> Self {
        Self {
            buffer,
            position: 0,
        }
    }
}

impl<'a> Serializer for Writer<'a> {
    fn serialize_string(&mut self, value: &str) { unimplemented!() }
    fn serialize_u8(&mut self, value: &u8) { unimplemented!() }
    fn serialize_i8(&mut self, value: &i8) { unimplemented!() }
    fn serialize_u16(&mut self, value: &u16) { unimplemented!() }
    fn serialize_i16(&mut self, value: &i16) { unimplemented!() }
    fn serialize_u32(&mut self, value: &u32) { unimplemented!() }
    fn serialize_i32(&mut self, value: &i32) { unimplemented!() }
    fn serialize_u64(&mut self, value: &u64) { unimplemented!() }
    fn serialize_i64(&mut self, value: &i64) { unimplemented!() }
    fn serialize_f32(&mut self, value: &f32) { unimplemented!() }
    fn serialize_f64(&mut self, value: &f64) { unimplemented!() }
}

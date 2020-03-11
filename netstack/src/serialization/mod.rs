mod reader;
mod writer;
mod zig_zag;
mod error;
pub use reader::Reader;
pub use writer::Writer;
pub use error::SerializationError;

macro_rules! impl_deserialize {
    ($t:ident, $d:tt) => {
        impl Deserialize for $t {
            type Item = $t;

            fn deserialize(deserializer: &mut impl Deserializer) -> Result<Self::Item, SerializationError> {
                deserializer.$d()
            }
        }
    }
}

macro_rules! impl_serialize {
    ($t:ident, $d:tt) => {
        impl Serialize for $t {
            fn serialize(&self, serializer: &mut impl Serializer) {
                serializer.$d(&self);
            }
        }
    }
}

impl_deserialize!(String, deserialize_string);
impl_serialize!(String, serialize_string);

impl_deserialize!(u8, deserialize_u8);
impl_serialize!(u8, serialize_u8);
impl_deserialize!(i8, deserialize_i8);
impl_serialize!(i8, serialize_i8);

impl_deserialize!(u16, deserialize_u16);
impl_serialize!(u16, serialize_u16);
impl_deserialize!(i16, deserialize_i16);
impl_serialize!(i16, serialize_i16);

impl_deserialize!(u32, deserialize_u32);
impl_serialize!(u32, serialize_u32);
impl_deserialize!(i32, deserialize_i32);
impl_serialize!(i32, serialize_i32);

impl_deserialize!(u64, deserialize_u64);
impl_serialize!(u64, serialize_u64);
impl_deserialize!(i64, deserialize_i64);
impl_serialize!(i64, serialize_i64);

impl_deserialize!(f32, deserialize_f32);
impl_serialize!(f32, serialize_f32);

impl_deserialize!(f64, deserialize_f64);
impl_serialize!(f64, serialize_f64);

pub trait Deserialize {
    type Item;

    fn deserialize(deserializer: &mut impl Deserializer) -> Result<Self::Item, SerializationError>;
}

pub trait Serialize {
    fn serialize(&self, serializer: &mut impl Serializer);
}

pub trait Deserializer {
    fn deserialize_string(&mut self) -> Result<String, SerializationError>;
    fn deserialize_u8(&mut self) -> Result<u8, SerializationError>;
    fn deserialize_i8(&mut self) -> Result<i8, SerializationError>;
    fn deserialize_u16(&mut self) -> Result<u16, SerializationError>;
    fn deserialize_i16(&mut self) -> Result<i16, SerializationError>;
    fn deserialize_u32(&mut self) -> Result<u32, SerializationError>;
    fn deserialize_i32(&mut self) -> Result<i32, SerializationError>;
    fn deserialize_u64(&mut self) -> Result<u64, SerializationError>;
    fn deserialize_i64(&mut self) -> Result<i64, SerializationError>;
    fn deserialize_f32(&mut self) -> Result<f32, SerializationError>;
    fn deserialize_f64(&mut self) -> Result<f64, SerializationError>;
}

pub trait Serializer {
    fn serialize_string(&mut self, value: &str);
    fn serialize_u8(&mut self, value: &u8);
    fn serialize_i8(&mut self, value: &i8);
    fn serialize_u16(&mut self, value: &u16);
    fn serialize_i16(&mut self, value: &i16);
    fn serialize_u32(&mut self, value: &u32);
    fn serialize_i32(&mut self, value: &i32);
    fn serialize_u64(&mut self, value: &u64);
    fn serialize_i64(&mut self, value: &i64);
    fn serialize_f32(&mut self, value: &f32);
    fn serialize_f64(&mut self, value: &f64);
}

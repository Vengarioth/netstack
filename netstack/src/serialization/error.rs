#[derive(Debug, Fail)]
pub enum SerializationError {
    #[fail(display = "Parsing VarUInt further would overflow the maximum number of iterations")]
    VarUIntOverflow,
}

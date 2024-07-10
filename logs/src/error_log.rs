#[derive(Debug)]
pub enum ErrorLog {
    UDPSocketError(String),
    SerializationError(String),
    FileError(String),
}

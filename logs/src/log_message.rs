use super::{error_log::ErrorLog, sources::Sources};

use std::io::Write;

pub struct LogMessage {
    pub source: Sources,
    pub message: String,
}

impl LogMessage {
    pub fn new(source: Sources, message: String) -> Self {
        LogMessage { source, message }
    }

    pub fn serialize(&self, stream: &mut dyn Write) -> Result<(), ErrorLog> {
        self.source.serialize(stream)?;
        let numbre_of_bytes = self.message.len() as u32;

        if stream.write_all(&numbre_of_bytes.to_be_bytes()).is_err() {
            return Err(ErrorLog::SerializationError(
                "Error serializing message len to buffer".to_string(),
            ));
        }

        if stream.write_all(self.message.as_bytes()).is_err() {
            return Err(ErrorLog::SerializationError(
                "Error serializing message to buffer".to_string(),
            ));
        }
        Ok(())
    }

    pub fn deserialize(stream: &mut dyn std::io::Read) -> Result<LogMessage, ErrorLog> {
        let source = Sources::deserialize(stream)?;

        let mut buffer = [0; 4];
        if stream.read_exact(&mut buffer).is_err() {
            return Err(ErrorLog::SerializationError(
                "Error deserializing message len from buffer".to_string(),
            ));
        }
        let number_of_bytes = u32::from_be_bytes(buffer) as usize;

        let mut message = vec![0; number_of_bytes];
        if stream.read_exact(&mut message).is_err() {
            return Err(ErrorLog::SerializationError(
                "Error deserializing message from buffer".to_string(),
            ));
        }
        let message = String::from_utf8_lossy(&message).to_string();

        Ok(LogMessage::new(source, message))
    }
}

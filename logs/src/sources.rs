use crate::error_log::ErrorLog;
use std::io::Write;

pub enum Sources {
    Robot(u8),
    Screen(u8),
    Gateway,
}

impl std::fmt::Display for Sources {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Sources::Robot(id) => write!(f, "ROBOT({})", id),
            Sources::Screen(id) => write!(f, "SCREEN({})", id),
            Sources::Gateway => write!(f, "GATEWAY"),
        }
    }
}

impl Sources {
    pub fn serialize(&self, stream: &mut dyn Write) -> Result<(), ErrorLog> {
        let value: [u8; 2] = match self {
            Sources::Robot(id) => [0, *id],
            Sources::Screen(id) => [1, *id],
            Sources::Gateway => [2, 0],
        };
        if stream.write_all(&value[..]).is_err() {
            return Err(ErrorLog::SerializationError(
                "Error serializing source to buffer".to_string(),
            ));
        }
        Ok(())
    }

    pub fn deserialize(stream: &mut dyn std::io::Read) -> Result<Sources, ErrorLog> {
        let mut buffer = [0; 2];
        if stream.read_exact(&mut buffer).is_err() {
            return Err(ErrorLog::SerializationError(
                "Error deserializing source from buffer".to_string(),
            ));
        }
        match buffer[0] {
            0 => Ok(Sources::Robot(buffer[1])),
            1 => Ok(Sources::Screen(buffer[1])),
            2 => Ok(Sources::Gateway),
            _ => Err(ErrorLog::SerializationError(
                "Error deserializing source from buffer".to_string(),
            )),
        }
    }
}

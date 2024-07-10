use std::{io::Write, net::UdpSocket};

use super::{error_log::ErrorLog, log_message::LogMessage};

pub struct LoggerReceiver<W: Write> {
    port: u16,
    output: W,
    display_in_terminal: bool,
}

impl<W: Write> LoggerReceiver<W> {
    pub fn new(port: u16, output: W, display_in_terminal: bool) -> Self {
        LoggerReceiver {
            port,
            output,
            display_in_terminal,
        }
    }

    fn create_socket(&self) -> Result<UdpSocket, ErrorLog> {
        UdpSocket::bind(format!("localhost:{}", self.port))
            .map_err(|_| ErrorLog::UDPSocketError("Error creando el socket".to_string()))
    }

    fn receive_message(socket: &UdpSocket) -> Result<LogMessage, ErrorLog> {
        let mut buffer = [0; 1024];
        let (number_of_bytes, _) = socket
            .recv_from(&mut buffer)
            .map_err(|_| ErrorLog::UDPSocketError("Error recibiendo mensaje".to_string()))?;

        LogMessage::deserialize(&mut &buffer[..number_of_bytes])
    }

    fn format_message(message: LogMessage) -> String {
        format!("[{}] {}", message.source, message.message)
    }

    pub fn receive_logs(&mut self) -> Result<(), ErrorLog> {
        let socket = self.create_socket()?;

        loop {
            let message = Self::receive_message(&socket)?;
            let text = Self::format_message(message);

            if self.display_in_terminal {
                println!("{}", text);
            }

            if self.output.write_all(text.as_bytes()).is_err() {
                return Err(ErrorLog::FileError(
                    "Error escribiendo en el archivo".to_string(),
                ));
            }
        }
    }
}

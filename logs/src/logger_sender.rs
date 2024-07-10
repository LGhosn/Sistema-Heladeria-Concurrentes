use std::net::UdpSocket;

use super::{error_log::ErrorLog, log_message::LogMessage, sources::Sources};

pub struct LoggerSender {
    receiver_port: u16,
    socket: UdpSocket,
}

impl LoggerSender {
    pub fn new(receiver_port: u16) -> Result<Self, ErrorLog> {
        let socket = UdpSocket::bind("localhost:0")
            .map_err(|_| ErrorLog::UDPSocketError("Error creando el socket".to_string()))?;

        Ok(LoggerSender {
            receiver_port,
            socket,
        })
    }

    pub fn send_log(&self, message: LogMessage) -> Result<(), ErrorLog> {
        let mut serialized_message = Vec::new();
        message.serialize(&mut serialized_message)?;

        let receiver_address = format!("localhost:{}", self.receiver_port);

        self.socket
            .send_to(&serialized_message, receiver_address)
            .map_err(|_| ErrorLog::UDPSocketError("Error enviando mensaje".to_string()))?;

        Ok(())
    }

    pub fn send_robot_log(&self, id: u8, message: String) -> Result<(), ErrorLog> {
        self.send_log(LogMessage::new(Sources::Robot(id), message))
    }

    pub fn send_screen_log(&self, id: u8, message: String) -> Result<(), ErrorLog> {
        self.send_log(LogMessage::new(Sources::Screen(id), message))
    }

    pub fn send_gateway_log(&self, message: String) -> Result<(), ErrorLog> {
        self.send_log(LogMessage::new(Sources::Gateway, message))
    }
}

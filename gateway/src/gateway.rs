use ice_cream_shop::messages::gateway_msg::GatewayMsg;
use std::{
    io::{self, Read, Write},
    net::TcpListener,
    thread,
};

/// Struct that represents a gateway that will receive messages from the screen.
pub struct Gateway {
    port: u16,
}

impl Gateway {
    /// Creates a new gateway with the given port.
    pub fn new(port: u16) -> Self {
        Gateway { port }
    }

    /// Function that validates if the order is valid.
    ///
    /// # Arguments
    ///
    /// * `credit_card` - The credit card number to validate.
    ///
    /// # Returns
    ///
    /// A boolean indicating if the order is valid.
    fn is_order_valid(credit_card: &str) -> bool {
        credit_card
            .chars()
            .next()
            .filter(|num| num.is_numeric())
            .map(|num| num != '3')
            .unwrap_or_default()
    }

    /// Function that handles the screen messages.
    /// It will read the messages from the stream and print the corresponding message.
    ///
    /// # Arguments
    ///
    /// * `stream` - The stream to read the messages from.
    ///
    /// # Returns
    ///
    /// An io::Result indicating if the function was successful.
    fn handle_screen<T: Read + Write>(mut stream: T) -> io::Result<()> {
        let mut buffer = [0; 256];

        while let Ok(n) = stream.read(&mut buffer) {
            if n == 0 {
                break;
            }

            match serde_json::from_slice(&buffer[..n]) {
                Ok(GatewayMsg::CapturePayment(order_id, credit_card)) => {
                    let response = Self::is_order_valid(&credit_card);

                    let print_msg = match response {
                        true => format!("Capturing payment for order {order_id:?}"),
                        false => format!("Invalid credit card for order {order_id:?}"),
                    };

                    println!("{print_msg}");
                    stream.write_all(response.to_string().as_bytes())?;
                }

                Ok(GatewayMsg::CommitPayment(order_id)) => {
                    println!("Committing payment for order {order_id:?}");
                }

                Ok(GatewayMsg::CancelPayment(order_id)) => {
                    println!("Cancelling payment for order {order_id:?}");
                }

                Err(e) => eprintln!("{e}"),
            }
        }

        Ok(())
    }

    /// Function that listens with a TCP listener for new connections and spawns a new thread for each one.
    /// Each thread will handle the screen messages.
    ///
    /// # Returns
    ///
    /// An io::Result indicating if the function was successful.
    pub fn receive_messages(&self) -> io::Result<()> {
        let listener = TcpListener::bind(format!("127.0.0.1:{}", self.port))?;
        println!("Ready to rumble!!!");

        for stream in listener.incoming().flatten() {
            thread::spawn(move || Self::handle_screen(stream));
        }

        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use ice_cream_shop::{io_err, orders::OrderId};
    use std::{cmp, convert::identity};

    struct MockStream {
        pub data: Vec<u8>,
        can_read: bool,
    }

    impl Read for MockStream {
        fn read(&mut self, buf: &mut [u8]) -> std::io::Result<usize> {
            if !self.can_read {
                return Err(io_err!("MockStream"));
            }

            let len = cmp::min(buf.len(), self.data.len());
            buf[..len].copy_from_slice(&self.data[..len]);
            self.data = self.data[len..].to_vec();
            self.can_read = false;
            Ok(len)
        }
    }

    impl Write for MockStream {
        fn write(&mut self, buf: &[u8]) -> std::io::Result<usize> {
            self.data.extend_from_slice(buf);
            Ok(buf.len())
        }

        fn flush(&mut self) -> std::io::Result<()> {
            Ok(())
        }
    }

    fn capture_payment(order_id: OrderId, credit_card_number: &str) -> io::Result<bool> {
        let req = GatewayMsg::CapturePayment(order_id, credit_card_number.to_string());
        let bytes = serde_json::to_string(&req)?;

        let mut mock_stream = MockStream {
            data: bytes.into_bytes(),
            can_read: true,
        };

        Gateway::handle_screen(&mut mock_stream).unwrap();
        let res = String::from_utf8(mock_stream.data).unwrap();

        Ok(res.parse().unwrap())
    }

    #[test]
    fn test01_a_valid_credit_card_returns_a_valid_response() {
        let response = capture_payment(OrderId::new(3, 10), "1234");
        assert!(response.is_ok_and(identity))
    }

    #[test]
    fn test02_an_invalid_credit_card_returns_an_invalid_response() {
        let response = capture_payment(OrderId::new(3, 10), "3234");
        assert!(response.is_ok_and(|valid| !valid));
    }

    #[test]
    fn test03_an_empty_credit_card_returns_an_invalid_response() {
        let response = capture_payment(OrderId::new(3, 10), "");
        assert!(response.is_ok_and(|valid| !valid));
    }
}

use ice_cream_shop::{
    id_to_addr, io_err,
    messages::{fmt_msg, gateway_msg::GatewayMsg, robot_msg::RobotMsg},
    orders::{ClientOrder, Order, OrderId},
    shop_values::{N_ROBOTS, ROBOT_SCREEN_STARTING_PORT},
};
use std::{
    io::{self, BufRead, Read, Write},
    net::TcpStream,
};

/// Function that takes a BufRead and returns an iterator of ClientOrders.
pub fn orders<R: BufRead>(reader: R) -> impl Iterator<Item = ClientOrder> {
    reader
        .lines()
        .map_while(Result::ok)
        .flat_map(|line| serde_json::from_str(&line))
}

/// Struct that represents a screen that will communicate with the gateway and the robots.
#[derive(Clone)]
pub struct Screen {
    pub id: u16,
}

impl Screen {
    /// Creates a new screen with the given id.
    pub fn new(id: u16) -> Self {
        Self { id }
    }

    /// Validates the given order with the gateway.
    ///
    /// # Arguments
    ///
    /// * `order` - The order to validate.
    /// * `order_number` - The number of the order.
    /// * `gateway` - The gateway to communicate with.
    ///
    /// # Returns
    ///
    /// A boolean indicating if the order is valid.
    pub fn validate<T: Read + Write>(
        &self,
        order: &ClientOrder,
        order_number: usize,
        gateway: &mut T,
    ) -> io::Result<bool> {
        let card_number = order.card_number.to_string();
        let order = OrderId::new(self.id, order_number);
        let msg = fmt_msg(GatewayMsg::CapturePayment(order, card_number));
        gateway.write_all(&msg)?;

        let mut buffer = [0; 128];
        let n = gateway.read(&mut buffer)?;

        String::from_utf8_lossy(&buffer[..n])
            .parse()
            .map_err(|_| io_err!("Invalid gateway response"))
    }

    /// Commits the given order with the gateway.
    ///
    /// # Arguments
    ///
    /// * `order_number` - The number of the order.
    /// * `gateway` - The gateway to communicate with.
    ///
    /// # Returns
    ///
    /// An io::Result indicating if the commit was successful.
    pub fn commit(&self, order_number: usize, gateway: &mut TcpStream) -> io::Result<()> {
        let order = OrderId::new(self.id, order_number);
        let msg = fmt_msg(GatewayMsg::CommitPayment(order));
        gateway.write_all(&msg)
    }

    /// Cancels the given order with the gateway.
    ///
    /// # Arguments
    ///
    /// * `order_number` - The number of the order.
    /// * `gateway` - The gateway to communicate with.
    ///
    /// # Returns
    ///
    /// An io::Result indicating if the cancel was successful.
    pub fn cancel(&self, order_number: usize, gateway: &mut TcpStream) -> io::Result<()> {
        let order = OrderId::new(self.id, order_number);
        let msg = fmt_msg(GatewayMsg::CancelPayment(order));
        gateway.write_all(&msg)
    }

    /// Notifies the given order to the robots.
    ///
    /// # Arguments
    ///
    /// * `order` - The order to notify.
    ///
    /// # Returns
    ///
    /// An io::Result indicating if the notification was successful.
    pub fn notify_order(&self, order: Order) -> io::Result<()> {
        let order_number = order.id().order_number();
        let order = fmt_msg(RobotMsg::RecvOrder(order));

        for offset in 0..N_ROBOTS {
            let id = (order_number as u16 + offset) % N_ROBOTS;
            let ip = id_to_addr(ROBOT_SCREEN_STARTING_PORT, id);

            if let Ok(mut stream) = TcpStream::connect(ip) {
                if stream.write_all(&order).is_ok() {
                    return Ok(());
                }
            }
        }

        Err(io_err!("Could not notify any robot"))
    }
}

#[cfg(test)]

mod tests {
    use super::*;
    use ice_cream_shop::flavour::Flavour;
    use std::cmp;

    struct MockStream {
        pub data: Vec<u8>,
    }

    impl Read for MockStream {
        fn read(&mut self, buf: &mut [u8]) -> std::io::Result<usize> {
            let len = cmp::min(buf.len(), self.data.len());
            buf[..len].copy_from_slice(&self.data[..len]);
            self.data = self.data[len..].to_vec();
            Ok(len)
        }
    }

    impl Write for MockStream {
        fn write(&mut self, buf: &[u8]) -> std::io::Result<usize> {
            //self.data.extend_from_slice(buf);
            Ok(buf.len())
        }

        fn flush(&mut self) -> std::io::Result<()> {
            Ok(())
        }
    }

    #[test]
    fn test01_i_can_validate_a_valid_order() {
        let client_order = ClientOrder {
            flavours: vec![Flavour::Chocolate, Flavour::DulceDeLeche]
                .into_iter()
                .map(|flavour| (flavour, 1))
                .collect(),
            card_number: "6666-1111-2222-3333".to_string(),
        };

        let screen = Screen::new(1);

        let mut mock_stream = MockStream {
            data: "true".to_string().into_bytes(),
        };

        let result = screen.validate(&client_order, 1, &mut mock_stream);
        assert_eq!(result.unwrap(), true);
    }

    #[test]
    fn test02_i_can_validate_an_invalid_order() {
        let client_order = ClientOrder {
            flavours: vec![Flavour::Chocolate, Flavour::DulceDeLeche]
                .into_iter()
                .map(|flavour| (flavour, 1))
                .collect(),
            card_number: "3666-1111-2222-3333".to_string(),
        };

        let screen = Screen::new(1);

        let mut mock_stream = MockStream {
            data: "false".to_string().into_bytes(),
        };

        let result = screen.validate(&client_order, 1, &mut mock_stream);
        assert_eq!(result.unwrap(), false);
    }
}

// #[cfg(test)]
// mod tests {
//     use super::*;
//     use ice_cream_shop::ice_cream::{Cup, Flavour};

//     /*
//     PARA CORRER LOS TESTS PRIMERO HAY QUE INICIAR EL GATEWAY
//     */
//     #[test]
//     fn validate_correct_card_number() {
//         let order = ClientOrder {
//             card_number: "1234".to_string(),
//             flavours: vec![(Flavour::Chocolate, 1), (Flavour::BananaSplit, 2)]
//                 .into_iter()
//                 .collect(),
//             cup_size: Cup::Small,
//         };

//         let mut screen = Screen::new(0).unwrap();
//         let is_valid = screen.validate(&order, 0).unwrap();
//         assert!(is_valid);
//     }

//     #[test]
//     fn validate_incorrect_card_number() {
//         let order = ClientOrder {
//             card_number: "31234".to_string(),
//             flavours: vec![(Flavour::Chocolate, 1), (Flavour::BananaSplit, 2)]
//                 .into_iter()
//                 .collect(),
//             cup_size: Cup::Small,
//         };

//         let mut screen = Screen::new(0).unwrap();
//         let is_valid = screen.validate(&order, 0).unwrap();
//         assert!(!is_valid);
//     }

//     #[test]
//     fn commit_order() {
//         let order_number = 0;
//         let mut screen = Screen::new(0).unwrap();
//         let sent_commit = screen.commit(order_number);
//         assert!(sent_commit.is_ok());
//     }
// }

pub mod screen;

use ice_cream_shop::{
    id_to_addr,
    messages::screen_msg::ScreenMsg,
    orders::Order,
    shop_values::{GATEWAY_PORT, SCREEN_STARTING_PORT},
};
use screen::Screen;
use std::{
    convert::identity,
    fs::File,
    io::{self, BufRead, BufReader, Read},
    net::{TcpListener, TcpStream},
};

/// Function that receives messages from the gateway and different robots.
/// It will commit or cancel orders depending on the message received.
///
/// # Arguments
///
/// * `screen` - The screen that will receive the messages.
///
/// # Returns
///
/// An io::Result indicating if the function was successful.
pub fn receiver(screen: Screen) -> io::Result<()> {
    let mut gateway = TcpStream::connect(format!("127.0.0.1:{GATEWAY_PORT}"))?;
    let mut bytes = [0; 2048];

    let screen_ip = id_to_addr(SCREEN_STARTING_PORT, screen.id);
    let listener = TcpListener::bind(screen_ip)?;

    for mut stream in listener.incoming().flatten() {
        let Ok(n) = stream.read(&mut bytes) else {
            continue;
        };

        for line in bytes[..n].lines().map_while(Result::ok) {
            match serde_json::from_str(&line) {
                Ok(ScreenMsg::ConfirmOrder(order)) => {
                    println!(
                        "Order done: Screen {} - Order: {}",
                        order.screen_id(),
                        order.order_number()
                    );
                    screen.commit(order.order_number(), &mut gateway)?;
                }

                Ok(ScreenMsg::CancelOrder(order)) => {
                    println!(
                        "Order canceled: Screen {} - Order: {}",
                        order.screen_id(),
                        order.order_number()
                    );
                    screen.cancel(order.order_number(), &mut gateway)?;
                }

                Err(e) => eprintln!("Received an invalid message: {e}"),
            }
        }
    }

    Ok(())
}

/// Function that processes a file with orders and validates them with the gateway.
/// If the order is valid, it will notify the robots.
/// If the order is invalid, it will print a message.
///
/// # Arguments
///
/// * `orders_path` - The path to the file with the orders.
/// * `screen` - The screen that will validate the orders.
/// * `screen_id` - The id of the screen.
///
/// # Returns
///
/// An io::Result indicating if the function was successful.
pub fn process_file(orders_path: String, screen: Screen, screen_id: u16) -> io::Result<()> {
    let reader = BufReader::new(File::open(orders_path)?);
    let mut gateway = TcpStream::connect(format!("127.0.0.1:{GATEWAY_PORT}"))?;

    for (number, order) in screen::orders(reader).enumerate() {
        if screen
            .validate(&order, number, &mut gateway)
            .is_ok_and(identity)
        {
            println!("Order [{}] is valid", number);
            let order = Order::from(order, screen_id, number);
            screen.notify_order(order)?;
        } else {
            println!("Order [{}] is invalid", number);
        }
    }

    Ok(())
}

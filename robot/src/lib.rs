pub mod message;
pub mod robot;
pub mod token_box;

use actix::prelude::*;
use ice_cream_shop::messages::robot_msg::RobotMsg;
use message::*;
use robot::Robot;
use std::io::BufRead;
use tokio::{
    io::{AsyncReadExt, ReadHalf},
    net::{TcpListener, TcpStream},
};

/// Starts a TCP listener that will listen for robots that want to connect to the robot.
/// When they try to join, it sends a message to the robot for it to handle the connection.
async fn new_connections_receiver(robot_addr: Addr<Robot>, listener: TcpListener) {
    println!("0: new_connections_receiver started");

    while let Ok((stream, _)) = listener.accept().await {
        robot_addr.do_send(Connect { stream });
    }

    println!("0: new_connections_receiver ended");
}

/// Continuously reads from the stream of the robot that is before this robot in the chain
/// to receive messages from it. It will send the messages to the robot to handle them.
async fn prev_robot_receiver(robot: Addr<Robot>, mut stream: ReadHalf<TcpStream>) {
    println!("1: prev_robot_receiver started");

    let mut bytes = [0; 2048];
    while let Ok(n) = stream.read(&mut bytes).await {
        if n == 0 {
            break;
        }

        for line in bytes[..n].lines().map_while(Result::ok) {
            match serde_json::from_str(&line) {
                Ok(RobotMsg::RecvOrderToken(token)) => robot.do_send(RecvOrderToken { token }),
                Ok(RobotMsg::RecvFlavourToken(token)) => robot.do_send(RecvFlavourToken { token }),
                _ => eprintln!("Invalid message received at prev_robot_receiver"),
            }
        }
    }

    println!("1: prev_robot_receiver ended");
}

/// Continuously reads from the stream of the robot that is after this robot in the chain
/// to receive messages from it. It will send the messages to the robot to handle them.
async fn next_robot_receiver(robot: Addr<Robot>, mut stream: ReadHalf<TcpStream>) {
    println!("2: next_robot_receiver started");

    let mut bytes = [0; 2048];
    while let Ok(n) = stream.read(&mut bytes).await {
        if n == 0 {
            break;
        }

        for line in bytes[..n].lines().map_while(Result::ok) {
            match serde_json::from_str(&line) {
                Ok(RobotMsg::EndOfUse(token_id)) => robot.do_send(EndOfUse { token_id }),
                Ok(RobotMsg::Disconnect) => {
                    robot.do_send(FindNext);
                    println!("2: next_robot_receiver ended");
                    return;
                }

                _ => eprintln!("Invalid message received at next_robot_receiver"),
            }
        }
    }

    robot.do_send(FindNext);
    robot.do_send(CheckTokenBox);

    println!("2: next_robot_receiver ended");
}

/// Starts a TCP listener that will listen for orders that are sent to the robot.
/// When an order is received, it sends a message to the robot for it to handle the order.
async fn new_orders_receiver(robot_addr: Addr<Robot>, listener: TcpListener) {
    println!("3: new_orders_receiver started");

    let mut bytes = [0; 2048];
    while let Ok((mut stream, _)) = listener.accept().await {
        let Ok(n) = stream.read(&mut bytes).await else {
            continue;
        };

        match serde_json::from_slice(&bytes[..n]) {
            Ok(RobotMsg::RecvOrder(order)) => robot_addr.do_send(RecvOrder { order }),
            _ => eprintln!("Invalid message received at new_orders_receiver"),
        }
    }

    println!("3: new_orders_receiver ended");
}

pub mod gateway_msg;
pub mod robot_msg;
pub mod screen_msg;

use serde::Serialize;

/// Function that formats a message into a Vec<u8>
///
/// # Arguments
///
/// * `msg` - The message to format.
///
/// # Returns
///
/// A Vec<u8> with the formatted message.
pub fn fmt_msg<M: Serialize>(msg: M) -> Vec<u8> {
    let json = serde_json::to_string(&msg).unwrap_or_else(|e| e.to_string());
    format!("{json}\n").into_bytes()
}

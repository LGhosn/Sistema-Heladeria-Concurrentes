use crate::orders::OrderId;
use serde::{Deserialize, Serialize};

/// Enum that represents the messages that the screen can receive
#[derive(Serialize, Deserialize)]
pub enum ScreenMsg {
    ConfirmOrder(OrderId),
    CancelOrder(OrderId),
}

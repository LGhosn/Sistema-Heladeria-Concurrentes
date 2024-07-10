use crate::orders::OrderId;
use serde::{Deserialize, Serialize};

/// Enum that represents the messages that the gateway can receive
#[derive(Serialize, Deserialize)]
pub enum GatewayMsg {
    CapturePayment(OrderId, String),
    CommitPayment(OrderId),
    CancelPayment(OrderId),
}

use crate::{
    orders::Order,
    tokens::{FlavourToken, OrderToken, TokenId},
};
use serde::{Deserialize, Serialize};

/// Enum that represents the messages that the robot can receive
#[derive(Debug, Serialize, Deserialize)]
pub enum RobotMsg {
    // Prev
    RecvOrderToken(OrderToken),
    RecvFlavourToken(FlavourToken),

    // Next
    Disconnect,
    EndOfUse(TokenId),

    // Screen
    RecvOrder(Order),
}

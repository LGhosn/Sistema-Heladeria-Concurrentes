use actix::prelude::*;
use ice_cream_shop::{
    orders::Order,
    tokens::{FlavourToken, OrderToken, TokenId},
};
use tokio::net::TcpStream;

/// A message that tells the robot to check if it is alone in the token ring.
#[derive(Message, Debug)]
#[rtype(bool)]
pub struct IsAlone;

/// A message that tells the robot to update its prev_tx.
#[derive(Message, Debug)]
#[rtype(result = "()")]
pub struct Connect {
    pub stream: TcpStream,
}

/// A message that tells the robot to find a new robot to connect to (next_tx).
#[derive(Message, Debug)]
#[rtype(result = "()")]
pub struct FindNext;

// OrderToken

/// A message that tells the robot to receive an OrderToken.
#[derive(Message, Debug)]
#[rtype(result = "()")]
pub struct RecvOrderToken {
    pub token: OrderToken,
}

/// A message that tells the robot to release an OrderToken.
#[derive(Message, Debug)]
#[rtype(result = "()")]
pub struct ReleaseOrderToken {
    pub token: OrderToken,
}

// FlavourToken

/// A message that tells the robot to receive a FlavourToken.
#[derive(Message, Debug)]
#[rtype(result = "()")]
pub struct RecvFlavourToken {
    pub token: FlavourToken,
}

/// A message that tells the robot to release a FlavourToken.
#[derive(Message, Debug)]
#[rtype(result = "()")]
pub struct ReleaseFlavourToken {
    pub token: FlavourToken,
    // pub used: bool,
}

// TokenBox

/// A message that tells the robot that the next robot stopped using a token.
#[derive(Message, Debug)]
#[rtype(result = "()")]
pub struct EndOfUse {
    pub token_id: TokenId,
}

/// A message that tells the robot to check if it has any tokens to release.
#[derive(Message, Debug)]
#[rtype(result = "()")]
pub struct CheckTokenBox;

/// A message that tells the robot to receive a new order.
#[derive(Message, Debug)]
#[rtype(result = "()")]
pub struct RecvOrder {
    pub order: Order,
}

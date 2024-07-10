use crate::flavour::Flavour;
use serde::{Deserialize, Serialize};

/// Enum that represents the the types of tokens
#[derive(Serialize, Deserialize, Debug)]
pub enum TokenId {
    Order,
    Flavour(Flavour),
}

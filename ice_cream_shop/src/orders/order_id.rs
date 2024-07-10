use serde::{Deserialize, Serialize};

/// Struct that represents an order id
/// An order id is composed of a screen id and an order number.
#[derive(Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize, Debug)]
pub struct OrderId {
    screen_id: u16,
    order_number: usize,
}

impl OrderId {
    pub fn new(screen_id: u16, order_number: usize) -> Self {
        Self {
            screen_id,
            order_number,
        }
    }

    pub fn order_number(&self) -> usize {
        self.order_number
    }

    pub fn screen_id(&self) -> u16 {
        self.screen_id
    }
}

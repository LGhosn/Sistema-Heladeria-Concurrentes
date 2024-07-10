use super::{client_order::ClientOrder, order_id::OrderId};
use crate::flavour::Flavour;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;

/// Struct that represents an order the screen sends to the robots
///
/// # Attributes
///
/// * `id` - The id of the order.
/// * `flavours` - The flavours of the order.
/// * `cup_size` - The size of the cup.
#[derive(PartialEq, Eq, Serialize, Deserialize, Debug, Clone)]
pub struct Order {
    id: OrderId,
    flavours: HashMap<Flavour, usize>,
}

impl Order {
    /// Creates a new Order with the given client order, screen id and order number from a client order.
    ///
    /// # Arguments
    ///
    /// * `client_order` - The client order to create the order from.
    /// * `screen_id` - The id of the screen.
    /// * `order_number` - The number of the order.
    ///
    /// # Returns
    ///
    /// A new Order.
    pub fn from(client_order: ClientOrder, screen_id: u16, order_number: usize) -> Self {
        Self {
            id: OrderId::new(screen_id, order_number),
            flavours: client_order.flavours,
        }
    }

    /// Creates a new Order with the given id and flavours.
    ///
    /// # Arguments
    ///
    /// * `id` - The id of the order.
    /// * `flavours` - The flavours of the order.
    ///
    /// # Returns
    ///
    /// A new Order.
    pub fn new(id: OrderId, flavours: HashMap<Flavour, usize>) -> Self {
        Self { id, flavours }
    }

    /// Returns the id of the order.
    pub fn id(&self) -> OrderId {
        self.id
    }

    /// Returns the flavours of the order.
    ///
    /// # Arguments
    ///
    /// * `flavour` - The flavour to check.
    ///
    /// # Returns
    ///
    /// A boolean indicating if the order has the given flavour.
    pub fn has(&self, flavour: Flavour) -> bool {
        self.flavours.contains_key(&flavour)
    }

    /// Removes a flavour from the order.
    /// If the flavour is not in the order, it returns None.
    /// If the flavour is in the order, it returns the number of servings of the flavour.
    ///
    /// # Arguments
    ///
    /// * `flavour` - The flavour to remove.
    ///
    /// # Returns
    ///
    /// An Option containing the number of servings of the flavour.
    pub fn cross(&mut self, flavour: Flavour) -> Option<usize> {
        self.flavours.remove(&flavour)
    }

    /// Returns a boolean indicating if the order is completed.
    pub fn is_completed(&self) -> bool {
        self.flavours.is_empty()
    }
}

#[cfg(test)]

mod tests {
    use super::*;
    use crate::{
        flavour::Flavour,
        orders::{client_order::ClientOrder, order_id::OrderId},
    };

    #[test]
    fn test01_an_order_can_be_created_from_a_client_order() {
        let client_order = ClientOrder {
            flavours: vec![Flavour::Chocolate, Flavour::DulceDeLeche]
                .into_iter()
                .map(|flavour| (flavour, 1))
                .collect(),
            card_number: "0000-1111-2222-3333".to_string(),
        };
        let screen_id = 1;
        let order_number = 1;
        let order = Order::from(client_order, screen_id, order_number);
        assert_eq!(order.id(), OrderId::new(screen_id, order_number));
        assert_eq!(order.has(Flavour::Chocolate), true);
        assert_eq!(order.has(Flavour::DulceDeLeche), true);
    }

    #[test]
    fn test02flavours_can_be_removed_from_an_order() {
        let mut order = Order {
            id: OrderId::new(1, 1),
            flavours: vec![Flavour::Chocolate, Flavour::DulceDeLeche]
                .into_iter()
                .map(|flavour| (flavour, 1))
                .collect(),
        };
        assert_eq!(order.cross(Flavour::Chocolate), Some(1));
        assert_eq!(order.cross(Flavour::Chocolate), None);
        assert_eq!(order.cross(Flavour::DulceDeLeche), Some(1));
        assert_eq!(order.cross(Flavour::DulceDeLeche), None);
    }

    #[test]
    fn test03_an_order_without_flavours_is_completed() {
        let mut order = Order {
            id: OrderId::new(1, 1),
            flavours: vec![Flavour::Chocolate, Flavour::DulceDeLeche]
                .into_iter()
                .map(|flavour| (flavour, 1))
                .collect(),
        };
        assert_eq!(order.is_completed(), false);
        order.cross(Flavour::Chocolate);
        assert_eq!(order.is_completed(), false);
        order.cross(Flavour::DulceDeLeche);
        assert_eq!(order.is_completed(), true);
    }
}

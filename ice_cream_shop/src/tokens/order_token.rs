use crate::{orders::Order, tokens::TokenId};
use serde::{Deserialize, Serialize};
use std::collections::{HashMap, VecDeque};

/// Struct that represents the token that carries the different orders.
///
/// # Attributes
///
/// * `sender` - The id of the sender.
/// * `orders_queue` - The queue of orders.
/// * `in_progress` - The orders that are in progress.
#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct OrderToken {
    sender: u16,
    orders_queue: VecDeque<Order>,
    in_progress: HashMap<u16, Order>,
}

impl OrderToken {
    /// Creates a new OrderToken with the given id.
    ///
    /// # Arguments
    ///
    /// * `id` - The id of the token.
    ///
    /// # Returns
    ///
    /// A new OrderToken.
    pub fn new(id: u16) -> Self {
        Self {
            sender: id,
            orders_queue: VecDeque::new(),
            in_progress: HashMap::new(),
        }
    }

    /// Marks the token with the given id.
    /// This is used to know who sent the token.
    ///
    /// # Arguments
    ///
    /// * `id` - The id to mark the token with.
    pub fn mark(&mut self, id: u16) {
        self.sender = id;
    }

    /// Returns the id of the sender.
    ///
    /// # Returns
    ///
    /// The id of the sender.
    pub fn sender(&self) -> u16 {
        self.sender
    }

    /// Uploads the new orders to the queue.
    ///
    /// # Arguments
    ///
    /// * `orders` - The orders to upload.
    pub fn upload_new_orders(&mut self, orders: impl Iterator<Item = Order>) {
        for order in orders {
            self.orders_queue.push_back(order);
        }
    }

    /// Returns the next order in the queue.
    ///
    /// # Returns
    ///
    /// The next order in the queue, if there is any.
    pub fn next_order(&mut self) -> Option<Order> {
        self.orders_queue.pop_front()
    }

    /// Returns the id of the token.
    ///
    /// # Returns
    ///
    /// The id of the token.
    pub fn id(&self) -> TokenId {
        TokenId::Order
    }

    /// Adds an order to the in progress map.
    ///
    /// # Arguments
    ///
    /// * `robot_id` - The id of the robot.
    /// * `order` - The order to add.
    pub fn add_in_progress(&mut self, robot_id: u16, order: Order) {
        self.in_progress.insert(robot_id, order);
    }

    /// Removes an order from the in progress map.
    ///
    /// # Arguments
    ///
    /// * `robot_id` - The id of the robot.
    pub fn remove_in_progress(&mut self, robot_id: u16) -> Option<Order> {
        self.in_progress.remove(&robot_id)
    }
}

#[cfg(test)]

mod tests {
    use super::*;
    use crate::orders::{Order, OrderId};
    use std::collections::HashMap;

    #[test]
    fn test01_i_can_create_a_new_order_token() {
        let order_token = OrderToken::new(1);
        assert_eq!(order_token.sender(), 1);
        assert_eq!(order_token.orders_queue.len(), 0);
        assert_eq!(order_token.in_progress.len(), 0);
    }

    #[test]
    fn test02_i_can_mark_an_order_token() {
        let mut order_token = OrderToken::new(1);
        order_token.mark(2);
        assert_eq!(order_token.sender(), 2);
    }

    #[test]
    fn test03_i_can_add_new_orders_to_the_order_token() {
        let mut order_token = OrderToken::new(1);
        let order1 = Order::new(OrderId::new(1, 1), HashMap::new());
        let order2 = Order::new(OrderId::new(1, 2), HashMap::new());
        order_token.upload_new_orders(vec![order1.clone(), order2.clone()].into_iter());
        assert_eq!(order_token.orders_queue.len(), 2);
        assert_eq!(order_token.orders_queue.pop_front(), Some(order1));
        assert_eq!(order_token.orders_queue.pop_front(), Some(order2));
    }

    #[test]
    fn test04_i_can_get_the_next_order_from_the_order_token() {
        let mut order_token = OrderToken::new(1);
        let order1 = Order::new(OrderId::new(1, 1), HashMap::new());
        let order2 = Order::new(OrderId::new(1, 2), HashMap::new());
        order_token.upload_new_orders(vec![order1.clone(), order2.clone()].into_iter());
        assert_eq!(order_token.next_order(), Some(order1));
        assert_eq!(order_token.next_order(), Some(order2));
        assert_eq!(order_token.next_order(), None);
    }

    #[test]
    fn test05_i_can_add_an_order_in_progress_to_the_order_token() {
        let mut order_token = OrderToken::new(1);
        let order = Order::new(OrderId::new(1, 1), HashMap::new());
        order_token.add_in_progress(1, order.clone());
        assert_eq!(order_token.in_progress.len(), 1);
        assert_eq!(order_token.in_progress.get(&1), Some(&order));
    }

    #[test]
    fn test06_i_can_remove_an_order_in_progress_from_order_token() {
        let mut order_token = OrderToken::new(1);
        let order = Order::new(OrderId::new(1, 1), HashMap::new());
        order_token.add_in_progress(1, order.clone());
        assert_eq!(order_token.remove_in_progress(1), Some(order));
        assert_eq!(order_token.in_progress.len(), 0);
    }
}

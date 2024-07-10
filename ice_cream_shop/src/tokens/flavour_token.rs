use crate::{flavour::Flavour, tokens::TokenId};
use serde::{Deserialize, Serialize};

/// Struct that represents the token that carries a flavour
#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct FlavourToken {
    sender: u16,
    flavour: Flavour,
    servings: usize,
}

impl FlavourToken {
    /// Creates a new FlavourToken with the given id, flavour and servings.
    ///
    /// # Arguments
    ///
    /// * `id` - The id of the token.
    /// * `flavour` - The flavour of the token.
    /// * `servings` - The number of servings of the token.
    ///
    /// # Returns
    ///
    /// A new FlavourToken.
    pub fn new(id: u16, flavour: Flavour, servings: usize) -> Self {
        Self {
            sender: id,
            flavour,
            servings,
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

    /// Returns the flavour of the token.
    ///
    /// # Returns
    ///
    /// The flavour of the token.
    pub fn flavour(&self) -> Flavour {
        self.flavour
    }

    /// Returns the number of servings of the token.
    ///
    /// # Returns
    ///
    /// The number of servings of the token.
    pub fn servings(&self) -> usize {
        self.servings
    }

    /// Returns wheter or not a token has enough servings for an order.
    ///
    /// # Arguments
    ///
    /// * `servings` - The number of servings to check.
    ///
    /// # Returns
    ///
    /// A boolean indicating if the token has enough servings.
    pub fn has_enough(&self, servings: usize) -> bool {
        self.servings >= servings
    }

    /// Takes the given number of servings from the token.
    /// If the token has less servings than the given number, it will take all the servings.
    ///
    /// # Arguments
    ///
    /// * `servings` - The number of servings to take.
    ///
    /// # Returns
    ///
    /// The number of servings taken.
    pub fn take(&mut self, servings: usize) -> usize {
        let min = self.servings.min(servings);
        self.servings -= min;
        min
    }

    /// Returns the id of the token.
    ///
    /// # Returns
    ///
    /// The id of the token.
    pub fn id(&self) -> TokenId {
        TokenId::Flavour(self.flavour)
    }
}

#[cfg(test)]

mod tests {
    use super::*;

    #[test]
    fn test01_i_can_create_a_new_token() {
        let token = FlavourToken::new(1, Flavour::BananaSplit, 2);
        assert_eq!(token.sender(), 1);
        assert_eq!(token.flavour(), Flavour::BananaSplit);
        assert_eq!(token.servings(), 2);
    }

    #[test]
    fn test02_i_can_mark_the_token_with_my_id() {
        let mut token = FlavourToken::new(1, Flavour::BananaSplit, 2);
        token.mark(2);
        assert_eq!(token.sender(), 2);
    }

    #[test]
    fn test03_i_can_check_whether_the_token_has_enough_servings() {
        let token = FlavourToken::new(1, Flavour::BananaSplit, 2);
        assert!(token.has_enough(1));
        assert!(token.has_enough(2));
        assert!(!token.has_enough(3));
    }

    #[test]
    fn test04_i_can_take_servings_from_the_token() {
        let mut token = FlavourToken::new(1, Flavour::BananaSplit, 2);
        assert_eq!(token.take(1), 1);
        assert_eq!(token.servings(), 1);
        assert_eq!(token.take(2), 1);
        assert_eq!(token.servings(), 0);
        assert_eq!(token.take(1), 0);
        assert_eq!(token.servings(), 0);
    }
}

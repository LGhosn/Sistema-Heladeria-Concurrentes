use ice_cream_shop::{
    flavour::Flavour,
    tokens::{FlavourToken, OrderToken},
};
use std::collections::HashMap;

/// A struct that holds the tokens that the robot has sent to the next robot in the token ring
/// but that have not stopped being used by the next robot yet.
#[derive(Debug, Default)]
pub struct TokenBox {
    order_token: Option<OrderToken>,
    flavour_tokens: HashMap<Flavour, FlavourToken>,
}

impl TokenBox {
    /// Initializes a new TokenBox.
    pub fn new() -> Self {
        Self::default()
    }

    /// Returns true if the TokenBox has an OrderToken.
    ///
    /// # Arguments
    ///
    /// * `self` - The TokenBox.
    /// * `token` - The OrderToken.
    pub fn stash_order_token(&mut self, token: OrderToken) {
        self.order_token = Some(token);
    }

    /// Discards the OrderToken from the TokenBox.
    pub fn discard_order_token(&mut self) {
        self.order_token = None;
    }

    /// Takes the OrderToken from the TokenBox.
    pub fn take_order_token(&mut self) -> Option<OrderToken> {
        self.order_token.take()
    }

    /// Stashes a FlavourToken in the TokenBox.
    ///
    /// # Arguments
    ///
    /// * `self` - The TokenBox.
    /// * `token` - The FlavourToken.
    pub fn stash_flavour_token(&mut self, token: FlavourToken) {
        self.flavour_tokens.insert(token.flavour(), token);
    }

    /// Discards a FlavourToken from the TokenBox.
    ///
    /// # Arguments
    ///
    /// * `self` - The TokenBox.
    /// * `flavour` - The Flavour of the token to discard.
    pub fn discard_flavour_token(&mut self, flavour: Flavour) {
        let _ = self.flavour_tokens.remove(&flavour);
    }

    /// Takes all the FlavourTokens from the TokenBox.
    ///
    /// # Arguments
    ///
    /// * `self` - The TokenBox.
    ///
    /// # Returns
    ///
    /// An iterator over the FlavourTokens.
    pub fn take_flavour_tokens(&mut self) -> impl Iterator<Item = FlavourToken> + '_ {
        self.flavour_tokens.drain().map(|(_, token)| token)
    }
}

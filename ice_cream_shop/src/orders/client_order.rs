use crate::flavour::Flavour;
use serde::Deserialize;
use std::collections::HashMap;

/// Struct that represents the order of the client
///
/// # Attributes
///
/// * `flavours` - The flavours of the order.
/// * `card_number` - The card number of the client.
/// * `cup_size` - The size of the cup.
#[derive(Deserialize, Debug)]
pub struct ClientOrder {
    pub flavours: HashMap<Flavour, usize>,
    pub card_number: String,
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn json_format() {
        let s = r#"
        {
            "flavours": {
                "dulce_de_leche": 2,
                "banana_split": 1
            },
            "cup_size": "small",
            "card_number": "0000-1111-2222-3333"
        }"#;

        let _: ClientOrder = serde_json::from_str(s).unwrap();
    }
}

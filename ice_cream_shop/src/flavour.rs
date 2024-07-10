use serde::{Deserialize, Serialize};

#[derive(Deserialize, PartialEq, Eq, Hash, Debug, Serialize, Clone, Copy)]
#[serde(rename_all = "snake_case")]
pub enum Flavour {
    DulceDeLeche,
    BananaSplit,
    Chocolate,
    Frutilla,
    Menta,
}

impl Flavour {
    pub fn flavours() -> impl Iterator<Item = Self> {
        use Flavour as F;
        [
            F::DulceDeLeche,
            F::BananaSplit,
            F::Chocolate,
            F::Frutilla,
            F::Menta,
        ]
        .into_iter()
    }
}

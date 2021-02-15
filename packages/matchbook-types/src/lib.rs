use serde::{Deserialize, Serialize};

pub type Price = usize;
pub type Quantity = usize;
pub type SymbolOwned = String;
pub type SymbolRef<'a> = &'a str;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum Side {
    Bid,
    Ask,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum Command {
    LimitOrderSubmitRequest {
        side: Side,
        price: Price,
        quantity: Quantity,
        symbol: SymbolOwned,
    },
}

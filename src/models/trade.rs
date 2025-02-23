use rust_decimal::Decimal;
use serde::{Deserialize, Serialize};
use std::convert::TryFrom;
#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct Trade {
    pub id: u64,
    pub timestamp: f64, // Unix timestamp
    pub market: String,
    pub base_asset: String,
    pub quote_asset: String,
    pub price: Decimal,
    pub amount: Decimal,
    pub quote_amount: Decimal,

    pub ask_user_id: u32,
    pub ask_order_id: u64,
    pub ask_role: MarketRole, // Maker/Taker
    pub ask_fee: Decimal,

    pub bid_user_id: u32,
    pub bid_order_id: u64,
    pub bid_role: MarketRole,
    pub bid_fee: Decimal,
}

#[derive(Debug, Serialize, Deserialize, Clone, Copy, PartialEq)]
pub enum MarketRole {
    Maker, // Order was on the book and matched
    Taker, // Order was matched immediately
}

impl TryFrom<&str> for MarketRole {
    type Error = String;

    fn try_from(value: &str) -> Result<Self, Self::Error> {
        match value {
            v if v.eq_ignore_ascii_case("MAKER") => Ok(MarketRole::Maker),
            v if v.eq_ignore_ascii_case("TAKER") => Ok(MarketRole::Taker),
            _ => Err(format!("Invalid MarketRole: {}", value)),
        }
    }
}

impl From<MarketRole> for String {
    fn from(role: MarketRole) -> Self {
        match role {
            MarketRole::Maker => "MAKER".to_string(),
            MarketRole::Taker => "TAKER".to_string(),
        }
    }
}

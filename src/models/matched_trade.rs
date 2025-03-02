use bigdecimal::BigDecimal;
use serde::{Deserialize, Serialize};
use std::convert::TryFrom;
#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct MatchedTrade {
    pub id: String,
    pub timestamp: i64, // Unix timestamp
    pub market_id: String,
    pub price: BigDecimal,
    pub amount: BigDecimal,
    pub quote_amount: BigDecimal,

    pub taker_user_id: String,
    pub taker_order_id: String,
    pub taker_fee: BigDecimal,

    pub maker_user_id: String,
    pub maker_order_id: String,
    pub maker_fee: BigDecimal,
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

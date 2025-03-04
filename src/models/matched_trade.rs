use bigdecimal::BigDecimal;
use database::models::models::NewTrade;
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

impl From<MatchedTrade> for NewTrade {
    fn from(trade: MatchedTrade) -> Self {
        Self {
            id: trade.id,
            timestamp: trade.timestamp,
            market_id: trade.market_id,
            price: trade.price,
            amount: trade.amount,
            quote_amount: trade.quote_amount,
            taker_user_id: trade.taker_user_id,
            taker_order_id: trade.taker_order_id,
            taker_fee: trade.taker_fee,
            maker_user_id: trade.maker_user_id,
            maker_order_id: trade.maker_order_id,
            maker_fee: trade.maker_fee,
        }
    }
}

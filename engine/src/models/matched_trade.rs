use bigdecimal::BigDecimal;
use database::models::models::NewTrade;
use serde::{Deserialize, Serialize};
use std::convert::TryFrom;

#[derive(Debug, Serialize, Deserialize, Clone, Copy, PartialEq)]
pub enum TakerSide {
    Buy,
    Sell,
}
#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct MatchedTrade {
    pub id: String,
    pub timestamp: i64, // Unix timestamp
    pub market_id: String,
    pub price: BigDecimal,
    pub base_amount: BigDecimal,
    pub quote_amount: BigDecimal,

    pub seller_user_id: String,
    pub seller_order_id: String,
    pub seller_fee: BigDecimal,

    pub buyer_user_id: String,
    pub buyer_order_id: String,
    pub buyer_fee: BigDecimal,

    pub is_liquidation: bool,
    pub taker_side: String,
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
            base_amount: trade.base_amount,
            quote_amount: trade.quote_amount,
            seller_user_id: trade.seller_user_id,
            seller_order_id: trade.seller_order_id,
            seller_fee: trade.seller_fee,
            buyer_user_id: trade.buyer_user_id,
            buyer_order_id: trade.buyer_order_id,
            buyer_fee: trade.buyer_fee,
            is_liquidation: Some(trade.is_liquidation),
            taker_side: trade.taker_side.into(),
        }
    }
}

impl From<TakerSide> for String {
    fn from(side: TakerSide) -> Self {
        match side {
            TakerSide::Buy => "BUY".to_string(),
            TakerSide::Sell => "SELL".to_string(),
        }
    }
}

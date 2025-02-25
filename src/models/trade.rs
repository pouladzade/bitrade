use rust_decimal::Decimal;
use serde::{Deserialize, Serialize};
use std::convert::TryFrom;
#[derive(Debug, Serialize, Deserialize, Clone)]
/// Represents a trade in the market.
///
/// # Fields
/// 
/// * `id` - Unique identifier for the trade.
/// * `timestamp` - Unix timestamp of when the trade occurred.
/// * `market_id` - Identifier for the market where the trade took place.
/// * `base_asset` - The base asset involved in the trade.
/// * `quote_asset` - The quote asset involved in the trade.
/// * `price` - The price at which the trade was executed.
/// * `amount` - The amount of the base asset that was traded.
/// * `quote_amount` - The amount of the quote asset that was traded.
/// * `taker_user_id` - Identifier for the user who took the trade.
/// * `taker_order_id` - Identifier for the order placed by the taker.
/// * `taker_role` - Role of the taker in the market (Maker/Taker).
/// * `taker_fee` - Fee charged to the taker for the trade.
/// * `maker_user_id` - Identifier for the user who made the trade.
/// * `maker_order_id` - Identifier for the order placed by the maker.
/// * `maker_role` - Role of the maker in the market.
/// * `maker_fee` - Fee charged to the maker for the trade.
pub struct Trade {
    pub id: String,
    pub timestamp: i64, // Unix timestamp
    pub market_id: String,
    pub base_asset: String,
    pub quote_asset: String,
    pub price: Decimal,
    pub amount: Decimal,
    pub quote_amount: Decimal,

    pub taker_user_id: String,
    pub taker_order_id: String,
    pub taker_role: MarketRole, // Maker/Taker
    pub taker_fee: Decimal,

    pub maker_user_id: String,
    pub maker_order_id: String,
    pub maker_role: MarketRole,
    pub maker_fee: Decimal,
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

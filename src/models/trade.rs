use rust_decimal::Decimal;
use serde::{Deserialize, Serialize};

use super::{order::Order, verbose_state::VerboseTradeState};

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

    pub ask_order: Option<Order>,
    pub bid_order: Option<Order>,
    pub state_before: Option<VerboseTradeState>,
    pub state_after: Option<VerboseTradeState>,
}

#[derive(Debug, Serialize, Deserialize, Clone, Copy, PartialEq)]
pub enum MarketRole {
    Maker, // Order was on the book and matched
    Taker, // Order was matched immediately
}

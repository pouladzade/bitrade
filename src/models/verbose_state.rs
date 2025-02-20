use rust_decimal::Decimal;
use serde::{Deserialize, Serialize};

use super::order::OrderSide;
#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct VerboseOrderState {
    pub user_id: u32,
    pub order_id: u64,
    pub order_side: OrderSide,
    pub filled_base: Decimal,
    pub filled_quote: Decimal,
    pub filled_fee: Decimal,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct VerboseBalanceState {
    pub user_id: u32,
    pub asset: String,
    pub balance_total: Decimal, // balance_total = available + frozen
}

#[derive(Debug, Serialize, Deserialize, Clone, Default)]
pub struct VerboseTradeState {
    pub order_states: Vec<VerboseOrderState>,
    pub balance_states: Vec<VerboseBalanceState>,
}

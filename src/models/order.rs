use rust_decimal::Decimal;
use serde::{Deserialize, Serialize};
use std::cmp::Ordering;

#[derive(Serialize, Deserialize, Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord)]
pub enum OrderType {
    Limit,  // A limit order with a specific price
    Market, // A market order executed at the best available price
}

#[derive(Serialize, Deserialize, Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord)]
pub enum OrderSide {
    Buy,  // Bid order
    Sell, // Ask order
}
#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct Order {
    // Immutable order details
    pub id: u64,
    pub base_asset: String,    // Base currency (e.g., BTC)
    pub quote_asset: String,   // Quote currency (e.g., USDT)
    pub market: String,        // Market identifier (e.g., BTC/USDT)
    pub order_type: OrderType, // Limit, Market, etc.
    pub side: OrderSide,       // Buy or Sell
    pub user_id: u32,          // Owner of the order
    pub post_only: bool,       // True if maker-only
    pub price: Decimal,        // Order price
    pub amount: Decimal,       // Total amount

    // Fee structure
    pub maker_fee: Decimal, // Fee if executed as maker
    pub taker_fee: Decimal, // Fee if executed as taker

    pub create_time: f64, // Unix timestamp when order was created

    // Mutable order details
    pub remain: Decimal,        // Remaining unfilled amount
    pub frozen: Decimal,        // Frozen funds for the order
    pub filled_base: Decimal,   // Filled amount in base asset
    pub filled_quote: Decimal,  // Filled amount in quote asset
    pub filled_fee: Decimal,    // Accumulated fee paid
    pub update_time: f64,       // Last update timestamp
    pub partially_filled: bool, // Indicates if order is partially filled
}
impl PartialEq for Order {
    fn eq(&self, other: &Self) -> bool {
        self.id == other.id
    }
}

impl Eq for Order {}

impl PartialOrd for Order {
    fn partial_cmp(&self, other: &Self) -> Option<Ordering> {
        Some(self.cmp(other))
    }
}

impl Ord for Order {
    fn cmp(&self, other: &Self) -> Ordering {
        match (self.side, other.side) {
            (OrderSide::Buy, OrderSide::Buy) => {
                // For bids, higher price comes first
                other
                    .price
                    .cmp(&self.price)
                    .then_with(|| self.create_time.total_cmp(&other.create_time))
            }
            (OrderSide::Sell, OrderSide::Sell) => {
                // For asks, lower price comes first
                self.price
                    .cmp(&other.price)
                    .then_with(|| self.create_time.total_cmp(&other.create_time))
            }
            _ => panic!("Cannot compare orders with different sides"),
        }
    }
}

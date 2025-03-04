use bigdecimal::BigDecimal;
use serde::{Deserialize, Serialize};
use serde_json::from_str;
use std::cmp::Ordering;
use database::models::models::*;
use std::str::FromStr;
#[derive(Serialize, Deserialize, Debug, Clone)]
/// Represents an order in the trading system.
///
/// # Fields
///
/// * `id` - Unique order identifier.
/// * `base_asset` - Base currency (e.g., BTC).
/// * `quote_asset` - Quote currency (e.g., USDT).
/// * `market_id` - Market identifier (e.g., BTC/USDT).
/// * `order_type` - Type of the order (e.g., Limit, Market).
/// * `side` - Side of the order (Buy or Sell).
/// * `user_id` - Owner of the order.
/// * `price` - Order price.
/// * `amount` - Total amount of the order.
///
/// * `maker_fee` - Fee if executed as maker.
/// * `taker_fee` - Fee if executed as taker.
///
/// * `create_time` - Unix timestamp when the order was created.
///
/// * `remain` - Remaining unfilled amount.
/// * `frozen` - Frozen funds for the order.
/// * `filled_base` - Filled amount in base asset.
/// * `filled_quote` - Filled amount in quote asset.
/// * `filled_fee` - Accumulated fee paid.
/// * `update_time` - Last update timestamp.
/// * `partially_filled` - Indicates if the order is partially filled.
pub struct TradeOrder {
    // Immutable order details
    pub id: String,

    pub market_id: String,
    pub order_type: OrderType,
    pub side: OrderSide,
    pub user_id: String,
    pub price: BigDecimal,
    pub amount: BigDecimal,
    // Fee structure
    pub maker_fee: BigDecimal,
    pub taker_fee: BigDecimal,

    pub create_time: i64,
    // Mutable order details
    pub remain: BigDecimal,    
    pub filled_base: BigDecimal,
    pub filled_quote: BigDecimal,
    pub filled_fee: BigDecimal,
    pub update_time: i64,    
}

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

impl TryFrom<&str> for OrderType {
    type Error = String;

    fn try_from(value: &str) -> Result<Self, Self::Error> {
        match value.to_uppercase().as_str() {
            "LIMIT" => Ok(OrderType::Limit),
            "MARKET" => Ok(OrderType::Market),
            _ => Err(format!("Invalid OrderType: {}", value)),
        }
    }
}

impl From<OrderType> for String {
    fn from(order_type: OrderType) -> Self {
        match order_type {
            OrderType::Limit => "LIMIT".to_string(),
            OrderType::Market => "MARKET".to_string(),
        }
    }
}

impl TryFrom<&str> for OrderSide {
    type Error = String;

    fn try_from(value: &str) -> Result<Self, Self::Error> {
        match value.to_uppercase().as_str() {
            "BUY" => Ok(OrderSide::Buy),
            "SELL" => Ok(OrderSide::Sell),
            _ => Err(format!("Invalid OrderSide: {}", value)),
        }
    }
}

impl From<OrderSide> for String {
    fn from(order_side: OrderSide) -> Self {
        match order_side {
            OrderSide::Buy => "BUY".to_string(),
            OrderSide::Sell => "SELL".to_string(),
        }
    }
}

impl PartialEq for TradeOrder {
    fn eq(&self, other: &Self) -> bool {
        self.id == other.id
    }
}

impl Eq for TradeOrder {}

impl PartialOrd for TradeOrder {
    fn partial_cmp(&self, other: &Self) -> Option<Ordering> {
        Some(self.cmp(other))
    }
}

impl Ord for TradeOrder {
    fn cmp(&self, other: &Self) -> Ordering {
        match (self.side, other.side) {
            (OrderSide::Sell, OrderSide::Sell) => {
                // For bids, higher price comes first
                match other.price.cmp(&self.price) {
                    Ordering::Equal => self.create_time.cmp(&other.create_time), // Time priority
                    ordering => ordering,
                }
            }
            (OrderSide::Buy, OrderSide::Buy) => {
                // For asks, lower price comes first
                match self.price.cmp(&other.price) {
                    Ordering::Equal => self.create_time.cmp(&other.create_time), // Time priority
                    ordering => ordering,
                }
            }
            _ => panic!("Cannot compare orders with different sides"),
        }
    }
}

impl From<TradeOrder> for NewOrder {
    fn from(trade_order: TradeOrder) -> Self {
        let status = determine_order_status(&trade_order);
        Self {
            id: trade_order.id,
            market_id: trade_order.market_id,
            user_id: trade_order.user_id,
            order_type: trade_order.order_type.into(),
            side: trade_order.side.into(),
            price: trade_order.price,
            amount: trade_order.amount,
            maker_fee: trade_order.maker_fee,
            taker_fee: trade_order.taker_fee,
            create_time: trade_order.create_time,
            remain: trade_order.remain,
            filled_base: trade_order.filled_base,
            filled_quote: trade_order.filled_quote,
            filled_fee: trade_order.filled_fee,
            update_time: trade_order.update_time,
            status,
        }
    }
}

pub fn determine_order_status(trade_order: &TradeOrder) -> String {
    if trade_order.remain == BigDecimal::from(0) {
        "Filled".to_string()
    } else if trade_order.filled_base > BigDecimal::from(0) {
        "Partially Filled".to_string()
    } else {
        "Open".to_string()
    }
}
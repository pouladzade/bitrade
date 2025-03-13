use crate::models::trade_order::TradeOrder;
use bigdecimal::BigDecimal;
use database::persistence::Persistence;
use std::collections::{BinaryHeap, HashMap};
use std::sync::Arc;

#[derive(Debug, Clone)]
pub struct OrderBook<P>
where
    P: Persistence + 'static,
{
    bids: BinaryHeap<TradeOrder>, // Max-heap for bids (buy orders)
    asks: BinaryHeap<TradeOrder>, // Min-heap for asks (sell orders)
    bid_depth: HashMap<BigDecimal, BigDecimal>, // Price -> Total Amount
    ask_depth: HashMap<BigDecimal, BigDecimal>, // Price -> Total Amount
    persister: Arc<P>,
    market_price: Option<BigDecimal>,
    base_asset: String,
    quote_asset: String,
    market_id: String,
}

mod logger;
mod market_depth;
mod matching;
pub mod order_book;

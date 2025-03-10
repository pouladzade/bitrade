use crate::models::trade_order::{OrderSide, OrderType, TradeOrder};
use anyhow::Result;
use bigdecimal::BigDecimal;
use database::models::models::*;

// Model to represent a matched order in the trading system
#[derive(Debug, Clone)]
pub struct MatchedOrder {
    pub id: String,
    pub market_id: String,
    pub taker_order_id: String,
    pub maker_order_id: String,
    pub price: BigDecimal,
    pub base_amount: BigDecimal,
    pub quote_amount: BigDecimal,
    pub taker_fee: BigDecimal,
    pub maker_fee: BigDecimal,
    pub side: OrderSide,
    pub created_at: i64,
}

// Adaptor for TradeOrder conversions
pub struct TradeOrderAdaptor;

impl TradeOrderAdaptor {
    // Convert from database Order to our TradeOrder
    pub fn from_db_order(order: Order) -> Result<TradeOrder> {
        TradeOrder::try_from(order)
    }

    // Convert from our TradeOrder to database NewOrder
    pub fn to_db_order(trade_order: TradeOrder) -> NewOrder {
        trade_order.into()
    }
}

// Adaptor for MatchedOrder conversions
pub struct MatchedOrderAdaptor;

impl MatchedOrderAdaptor {
    // Convert from database MatchResult to our MatchedOrder
    pub fn from_db_match_result(match_result: MatchResult) -> Result<MatchedOrder> {
        Ok(MatchedOrder {
            id: match_result.id,
            market_id: match_result.market_id,
            taker_order_id: match_result.taker_order_id,
            maker_order_id: match_result.maker_order_id,
            price: match_result.price,
            base_amount: match_result.base_amount,
            quote_amount: match_result.quote_amount,
            taker_fee: match_result.taker_fee,
            maker_fee: match_result.maker_fee,
            side: OrderSide::try_from(match_result.side.as_str())
                .map_err(|e| anyhow::anyhow!("Invalid OrderSide: {}", e))?,
            created_at: match_result.created_at,
        })
    }

    // Convert from our MatchedOrder to database NewMatchResult
    pub fn to_db_match_result(matched_order: MatchedOrder) -> NewMatchResult {
        NewMatchResult {
            id: matched_order.id,
            market_id: matched_order.market_id,
            taker_order_id: matched_order.taker_order_id,
            maker_order_id: matched_order.maker_order_id,
            price: matched_order.price,
            base_amount: matched_order.base_amount,
            quote_amount: matched_order.quote_amount,
            taker_fee: matched_order.taker_fee,
            maker_fee: matched_order.maker_fee,
            side: matched_order.side.into(),
            created_at: matched_order.created_at,
        }
    }
}

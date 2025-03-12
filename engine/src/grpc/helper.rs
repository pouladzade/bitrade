use crate::grpc::spot::{AddOrderRequest, ProtoTrade};
use crate::models::{
    matched_trade::MatchedTrade,
    trade_order::{OrderSide, OrderType, TradeOrder},
};

use anyhow::{Context, Result};
use bigdecimal::{BigDecimal, Zero};
use common::utils::{get_utc_now, get_uuid_string};
use database::models::models::{OrderStatus, TimeInForce};
use std::str::FromStr;
use tonic::Status;

impl TryFrom<AddOrderRequest> for TradeOrder {
    type Error = anyhow::Error;

    fn try_from(req: AddOrderRequest) -> Result<Self> {
        let order_type = OrderType::try_from(req.order_type.as_str())
            .map_err(|e| Status::invalid_argument(format!("Invalid order type: {}", e)))?;

        let side = OrderSide::try_from(req.side.as_str())
            .map_err(|e| Status::invalid_argument(format!("Invalid order side: {}", e)))?;

        let price = BigDecimal::from_str(&req.price)
            .context("Failed to parse price as Decimal")
            .map_err(|e| Status::invalid_argument(e.to_string()))?;

        let base_amount = BigDecimal::from_str(&req.base_amount)
            .context("Failed to parse base amount as Decimal")
            .map_err(|e| Status::invalid_argument(e.to_string()))?;

        let quote_amount = BigDecimal::from_str(&req.quote_amount)
            .context("Failed to parse quote amount as Decimal")
            .map_err(|e| Status::invalid_argument(e.to_string()))?;

        let maker_fee = BigDecimal::from_str(&req.maker_fee)
            .context("Failed to parse maker fee as Decimal")
            .map_err(|e| Status::invalid_argument(e.to_string()))?;

        let taker_fee = BigDecimal::from_str(&req.taker_fee)
            .context("Failed to parse taker fee as Decimal")
            .map_err(|e| Status::invalid_argument(e.to_string()))?;

        Ok(TradeOrder {
            id: get_uuid_string(),
            market_id: req.market_id,
            order_type,
            side,
            user_id: req.user_id,
            price,
            base_amount: base_amount.clone(),
            quote_amount: quote_amount.clone(),
            maker_fee,
            taker_fee,
            create_time: get_utc_now(),
            client_order_id: Some(get_uuid_string()),
            expires_at: None,
            post_only: Some(false),
            remained_base: base_amount,
            remained_quote: quote_amount,
            filled_base: BigDecimal::zero(),
            filled_quote: BigDecimal::zero(),
            filled_fee: BigDecimal::zero(),
            update_time: get_utc_now(),
            time_in_force: Some(TimeInForce::GTC),
            status: OrderStatus::Open,
        })
    }
}

impl From<TradeOrder> for AddOrderRequest {
    fn from(order: TradeOrder) -> Self {
        AddOrderRequest {
            market_id: order.market_id,
            order_type: order.order_type.into(),
            side: order.side.into(),
            user_id: order.user_id,
            price: order.price.to_string(),
            base_amount: order.base_amount.to_string(),
            quote_amount: order.quote_amount.to_string(),
            maker_fee: order.maker_fee.to_string(),
            taker_fee: order.taker_fee.to_string(),
        }
    }
}

impl From<MatchedTrade> for ProtoTrade {
    fn from(trade: MatchedTrade) -> Self {
        ProtoTrade {
            id: trade.id,
            timestamp: trade.timestamp,
            market_id: trade.market_id,
            price: trade.price.to_string(),
            base_amount: trade.base_amount.to_string(),
            quote_amount: trade.quote_amount.to_string(),
            seller_user_id: trade.seller_user_id,
            seller_order_id: trade.seller_order_id,

            seller_fee: trade.seller_fee.to_string(),
            buyer_user_id: trade.buyer_user_id,
            buyer_order_id: trade.buyer_order_id,

            buyer_fee: trade.buyer_fee.to_string(),
        }
    }
}

impl From<&MatchedTrade> for ProtoTrade {
    fn from(trade: &MatchedTrade) -> Self {
        ProtoTrade {
            id: trade.id.clone(),
            timestamp: trade.timestamp,
            market_id: trade.market_id.clone(),
            price: trade.price.to_string(),
            base_amount: trade.base_amount.to_string(),
            quote_amount: trade.quote_amount.to_string(),
            seller_user_id: trade.seller_user_id.clone(),
            seller_order_id: trade.seller_order_id.clone(),
            seller_fee: trade.seller_fee.to_string(),
            buyer_user_id: trade.buyer_user_id.clone(),
            buyer_order_id: trade.buyer_order_id.clone(),
            buyer_fee: trade.buyer_fee.to_string(),
        }
    }
}

pub fn convert_trades(trades: Vec<MatchedTrade>) -> Vec<ProtoTrade> {
    trades.iter().map(ProtoTrade::from).collect()
}

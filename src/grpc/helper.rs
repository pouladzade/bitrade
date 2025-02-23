use std::str::FromStr;
use anyhow::{Result, anyhow};
use rust_decimal::Decimal;

use crate::models::{
    order::{Order, OrderSide, OrderType},
    trade::{MarketRole, Trade},
};
use crate::grpc::spot::{AddOrderRequest, ProtoTrade};


impl TryFrom<AddOrderRequest> for Order {
    type Error = anyhow::Error;

    fn try_from(req: AddOrderRequest) -> Result<Self> {
        Ok(Order {
            id: req.id,
            base_asset: req.base_asset,
            quote_asset: req.quote_asset,
            market: req.market,
            order_type: OrderType::try_from(req.order_type.as_str())
                .map_err(|_| anyhow!("Invalid order type: {}", req.order_type))?,
            side: OrderSide::try_from(req.side.as_str())
                .map_err(|_| anyhow!("Invalid order side: {}", req.side))?,
            user_id: req.user_id,
            post_only: req.post_only,
            price: Decimal::from_str(&req.price).map_err(|e| anyhow!("Invalid price format: {}", e))?,
            amount: Decimal::from_str(&req.amount).map_err(|e| anyhow!("Invalid amount format: {}", e))?,
            maker_fee: Decimal::from_str(&req.maker_fee).map_err(|e| anyhow!("Invalid maker fee format: {}", e))?,
            taker_fee: Decimal::from_str(&req.taker_fee).map_err(|e| anyhow!("Invalid taker fee format: {}", e))?,
            create_time: 0.0,
            remain: Decimal::from_str("0").unwrap_or(Decimal::ZERO),
            frozen: Decimal::from_str("0").unwrap_or(Decimal::ZERO),
            filled_base: Decimal::from_str("0").unwrap_or(Decimal::ZERO),
            filled_quote: Decimal::from_str("0").unwrap_or(Decimal::ZERO),
            filled_fee: Decimal::from_str("0").unwrap_or(Decimal::ZERO),
            update_time: 0.0,
            partially_filled: false,
        })
    }
}

impl From<Order> for AddOrderRequest {
    fn from(order: Order) -> Self {
        AddOrderRequest {
            id: order.id,
            base_asset: order.base_asset,
            quote_asset: order.quote_asset,
            market: order.market,
            order_type: order.order_type.into(),
            side: order.side.into(),
            user_id: order.user_id,
            post_only: order.post_only,
            price: order.price.to_string(),
            amount: order.amount.to_string(),
            maker_fee: order.maker_fee.to_string(),
            taker_fee: order.taker_fee.to_string(),
        }
    }
}

impl TryFrom<ProtoTrade> for Trade {
    type Error = anyhow::Error;

    fn try_from(proto: ProtoTrade) -> Result<Self> {
        Ok(Trade {
            id: proto.id,
            timestamp: proto.timestamp,
            market: proto.market,
            base_asset: proto.base_asset,
            quote_asset: proto.quote_asset,
            price: Decimal::from_str(&proto.price)
                .map_err(|e| anyhow!("Invalid price format: {}", e))?,
            amount: Decimal::from_str(&proto.amount)
                .map_err(|e| anyhow!("Invalid amount format: {}", e))?,
            quote_amount: Decimal::from_str(&proto.quote_amount)
                .map_err(|e| anyhow!("Invalid quote amount format: {}", e))?,
            ask_user_id: proto.ask_user_id,
            ask_order_id: proto.ask_order_id,
            ask_role: MarketRole::try_from(proto.ask_role.as_str())
                .map_err(|_| anyhow!("Invalid ask role: {}", proto.ask_role))?,
            ask_fee: Decimal::from_str(&proto.ask_fee)
                .map_err(|e| anyhow!("Invalid ask fee format: {}", e))?,
            bid_user_id: proto.bid_user_id,
            bid_order_id: proto.bid_order_id,
            bid_role: MarketRole::try_from(proto.bid_role.as_str())
                .map_err(|_| anyhow!("Invalid bid role: {}", proto.bid_role))?,
            bid_fee: Decimal::from_str(&proto.bid_fee)
                .map_err(|e| anyhow!("Invalid bid fee format: {}", e))?,
        })
    }
}

impl From<Trade> for ProtoTrade {
    fn from(trade: Trade) -> Self {
        ProtoTrade {
            id: trade.id,
            timestamp: trade.timestamp,
            market: trade.market,
            base_asset: trade.base_asset,
            quote_asset: trade.quote_asset,
            price: trade.price.to_string(),
            amount: trade.amount.to_string(),
            quote_amount: trade.quote_amount.to_string(),
            ask_user_id: trade.ask_user_id,
            ask_order_id: trade.ask_order_id,
            ask_role: trade.ask_role.into(),
            ask_fee: trade.ask_fee.to_string(),
            bid_user_id: trade.bid_user_id,
            bid_order_id: trade.bid_order_id,
            bid_role: trade.bid_role.into(),
            bid_fee: trade.bid_fee.to_string(),
        }
    }
}

impl From<&Trade> for ProtoTrade {
    fn from(trade: &Trade) -> Self {
        ProtoTrade {
            id: trade.id,
            timestamp: trade.timestamp,
            market: trade.market.clone(),
            base_asset: trade.base_asset.clone(),
            quote_asset: trade.quote_asset.clone(),
            price: trade.price.to_string(),
            amount: trade.amount.to_string(),
            quote_amount: trade.quote_amount.to_string(),
            ask_user_id: trade.ask_user_id,
            ask_order_id: trade.ask_order_id,
            ask_role: trade.ask_role.clone().into(),
            ask_fee: trade.ask_fee.to_string(),
            bid_user_id: trade.bid_user_id,
            bid_order_id: trade.bid_order_id,
            bid_role: trade.bid_role.clone().into(),
            bid_fee: trade.bid_fee.to_string(),
        }
    }
}

pub fn convert_trades(trades: Vec<Trade>) -> Vec<ProtoTrade> {
    trades.iter().map(ProtoTrade::from).collect()
}

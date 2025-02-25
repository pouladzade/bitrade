use crate::grpc::spot::{AddOrderRequest, ProtoTrade};
use crate::models::{
    order::{Order, OrderSide, OrderType},
    trade::{MarketRole, Trade},
};
use crate::utils;
use anyhow::{anyhow, Context, Result};
use rust_decimal::Decimal;
use std::str::FromStr;
use tonic::Status;

impl TryFrom<AddOrderRequest> for Order {
    type Error = anyhow::Error;

    fn try_from(req: AddOrderRequest) -> Result<Self> {
        let order_type = OrderType::try_from(req.order_type.as_str())
            .map_err(|e| Status::invalid_argument(format!("Invalid order type: {}", e)))?;

        let side = OrderSide::try_from(req.side.as_str())
            .map_err(|e| Status::invalid_argument(format!("Invalid order side: {}", e)))?;

        let mut price = Decimal::from_str(&req.price)
            .context("Failed to parse price as Decimal")
            .map_err(|e| Status::invalid_argument(e.to_string()))?;

        // For market orders, we adjust the price to extreme values:
        // - For a market buy order, we set the price to Decimal::MAX so that it matches
        //   against the lowest available ask price.
        // - For a market sell order, we set the price to Decimal::MIN so that it matches
        //   against the highest available bid price.
        // Note: The actual execution price will be determined during the matching process.
        match (order_type, side) {
            (OrderType::Market, OrderSide::Buy) => {
                price = Decimal::MAX;
            }
            (OrderType::Market, OrderSide::Sell) => {
                price = Decimal::MIN;
            }
            _ => {}
        }

        let amount = Decimal::from_str(&req.amount)
            .context("Failed to parse amount as Decimal")
            .map_err(|e| Status::invalid_argument(e.to_string()))?;

        let maker_fee = Decimal::from_str("0")
            .context("Failed to parse maker_fee as Decimal")
            .map_err(|e| Status::internal(e.to_string()))?;

        let taker_fee = Decimal::from_str("0")
            .context("Failed to parse taker_fee as Decimal")
            .map_err(|e| Status::internal(e.to_string()))?;

        Ok(Order {
            id: utils::generate_uuid_id(),
            base_asset: req.base_asset,
            quote_asset: req.quote_asset,
            market_id: req.market_id,
            order_type,
            side,
            user_id: req.user_id,
            price,
            amount,
            maker_fee,
            taker_fee,
            create_time: utils::get_utc_now_time_millisecond(),
            remain: amount,
            frozen: Decimal::ZERO,
            filled_base: Decimal::ZERO,
            filled_quote: Decimal::ZERO,
            filled_fee: Decimal::ZERO,
            update_time:  utils::get_utc_now_time_millisecond(),
            partially_filled: false,
        })
    }
}

impl From<Order> for AddOrderRequest {
    fn from(order: Order) -> Self {
        AddOrderRequest {
            base_asset: order.base_asset,
            quote_asset: order.quote_asset,
            market_id: order.market_id,
            order_type: order.order_type.into(),
            side: order.side.into(),
            user_id: order.user_id,
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
            market_id: proto.market_id,
            base_asset: proto.base_asset,
            quote_asset: proto.quote_asset,
            price: Decimal::from_str(&proto.price)
                .map_err(|e| anyhow!("Invalid price format: {}", e))?,
            amount: Decimal::from_str(&proto.amount)
                .map_err(|e| anyhow!("Invalid amount format: {}", e))?,
            quote_amount: Decimal::from_str(&proto.quote_amount)
                .map_err(|e| anyhow!("Invalid quote amount format: {}", e))?,
            taker_user_id: proto.taker_user_id,
            taker_order_id: proto.taker_order_id,
            taker_role: MarketRole::try_from(proto.taker_role.as_str())
                .map_err(|_| anyhow!("Invalid ask role: {}", proto.taker_role))?,
            taker_fee: Decimal::from_str(&proto.taker_fee)
                .map_err(|e| anyhow!("Invalid ask fee format: {}", e))?,
            maker_user_id: proto.maker_user_id,
            maker_order_id: proto.maker_order_id,
            maker_role: MarketRole::try_from(proto.maker_role.as_str())
                .map_err(|_| anyhow!("Invalid bid role: {}", proto.maker_role))?,
            maker_fee: Decimal::from_str(&proto.maker_fee)
                .map_err(|e| anyhow!("Invalid bid fee format: {}", e))?,
        })
    }
}

impl From<Trade> for ProtoTrade {
    fn from(trade: Trade) -> Self {
        ProtoTrade {
            id: trade.id,
            timestamp: trade.timestamp,
            market_id: trade.market_id,
            base_asset: trade.base_asset,
            quote_asset: trade.quote_asset,
            price: trade.price.to_string(),
            amount: trade.amount.to_string(),
            quote_amount: trade.quote_amount.to_string(),
            taker_user_id: trade.taker_user_id,
            taker_order_id: trade.taker_order_id,
            taker_role: trade.taker_role.into(),
            taker_fee: trade.taker_fee.to_string(),
            maker_user_id: trade.maker_user_id,
            maker_order_id: trade.maker_order_id,
            maker_role: trade.maker_role.into(),
            maker_fee: trade.maker_fee.to_string(),
        }
    }
}

impl From<&Trade> for ProtoTrade {
    fn from(trade: &Trade) -> Self {
        ProtoTrade {
            id: trade.id.clone(),
            timestamp: trade.timestamp,
            market_id: trade.market_id.clone(),
            base_asset: trade.base_asset.clone(),
            quote_asset: trade.quote_asset.clone(),
            price: trade.price.to_string(),
            amount: trade.amount.to_string(),
            quote_amount: trade.quote_amount.to_string(),
            taker_user_id: trade.taker_user_id.clone(),
            taker_order_id: trade.taker_order_id.clone(),
            taker_role: trade.taker_role.clone().into(),
            taker_fee: trade.taker_fee.to_string(),
            maker_user_id: trade.maker_user_id.clone(),
            maker_order_id: trade.maker_order_id.clone(),
            maker_role: trade.maker_role.clone().into(),
            maker_fee: trade.maker_fee.to_string(),
        }
    }
}

pub fn convert_trades(trades: Vec<Trade>) -> Vec<ProtoTrade> {
    trades.iter().map(ProtoTrade::from).collect()
}

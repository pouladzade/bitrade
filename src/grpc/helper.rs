use std::str::FromStr;

use rust_decimal::Decimal;

use crate::models::{
    order::{Order, OrderSide, OrderType},
    trade::{MarketRole, Trade},
};
use crate::grpc::spot::{AddOrderRequest, ProtoTrade};


impl TryFrom<AddOrderRequest> for Order {
    type Error = String;
    fn try_from(req: AddOrderRequest) -> Result<Self, Self::Error> {
        Ok(Order {
            id: req.id,
            base_asset: req.base_asset,
            quote_asset: req.quote_asset,
            market: req.market,
            order_type: OrderType::try_from(req.order_type).unwrap(),
            side: OrderSide::try_from(req.side).unwrap(),
            user_id: req.user_id,
            post_only: req.post_only,
            price: Decimal::from_str(&req.price).map_err(|_| "Invalid price format")?,
            amount: Decimal::from_str(&req.amount).map_err(|_| "Invalid amount format")?,
            maker_fee: Decimal::from_str(&req.maker_fee).map_err(|_| "Invalid maker fee format")?,
            taker_fee: Decimal::from_str(&req.taker_fee).map_err(|_| "Invalid taker fee format")?,
            create_time: 0.0,
            remain: Decimal::from_str("0").unwrap(),
            frozen: Decimal::from_str("0").unwrap(),
            filled_base: Decimal::from_str("0").unwrap(),
            filled_quote: Decimal::from_str("0").unwrap(),
            filled_fee: Decimal::from_str("0").unwrap(),
            update_time: 0.0,
            partially_filled: false,
        })
    }
}

impl Into<AddOrderRequest> for Order {
    fn into(self) -> AddOrderRequest {
        AddOrderRequest {
            id: self.id,
            base_asset: self.base_asset,
            quote_asset: self.quote_asset,
            market: self.market,
            order_type: self.order_type.into(),
            side: self.side.into(),
            user_id: self.user_id,
            post_only: self.post_only,
            price: self.price.to_string(),
            amount: self.amount.to_string(),
            maker_fee: self.maker_fee.to_string(),
            taker_fee: self.taker_fee.to_string(),
        }
    }
}

impl From<ProtoTrade> for Trade {
    fn from(proto: ProtoTrade) -> Self {
        Trade {
            id: proto.id,
            timestamp: proto.timestamp,
            market: proto.market,
            base_asset: proto.base_asset,
            quote_asset: proto.quote_asset,
            price: Decimal::from_str(&proto.price).unwrap_or(Decimal::ZERO),
            amount: Decimal::from_str(&proto.amount).unwrap_or(Decimal::ZERO),
            quote_amount: Decimal::from_str(&proto.quote_amount).unwrap_or(Decimal::ZERO),
            ask_user_id: proto.ask_user_id,
            ask_order_id: proto.ask_order_id,
            ask_role: MarketRole::try_from(proto.ask_role).unwrap(),
            ask_fee: Decimal::from_str(&proto.ask_fee).unwrap_or(Decimal::ZERO),
            bid_user_id: proto.bid_user_id,
            bid_order_id: proto.bid_order_id,
            bid_role: MarketRole::try_from(proto.bid_role).unwrap(),
            bid_fee: Decimal::from_str(&proto.bid_fee).unwrap_or(Decimal::ZERO),
        }
    }
}


impl Into<ProtoTrade> for Trade {
    fn into(self) -> ProtoTrade {
        ProtoTrade {
            id: self.id,
            timestamp: self.timestamp,
            market: self.market,
            base_asset: self.base_asset,
            quote_asset: self.quote_asset,
            price: self.price.to_string(),
            amount: self.amount.to_string(),
            quote_amount: self.quote_amount.to_string(),
            ask_user_id: self.ask_user_id,
            ask_order_id: self.ask_order_id,
            ask_role: self.ask_role.into(),
            ask_fee: self.ask_fee.to_string(),
            bid_user_id: self.bid_user_id,
            bid_order_id: self.bid_order_id,
            bid_role: self.bid_role.into(),
            bid_fee: self.bid_fee.to_string(),
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
            price: trade.price.to_string(),         // Convert Decimal to String
            amount: trade.amount.to_string(),       // Convert Decimal to String
            quote_amount: trade.quote_amount.to_string(),

            ask_user_id: trade.ask_user_id,
            ask_order_id: trade.ask_order_id,
            ask_role: trade.ask_role.clone().into(), // Convert MarketRole to String
            ask_fee: trade.ask_fee.to_string(),

            bid_user_id: trade.bid_user_id,
            bid_order_id: trade.bid_order_id,
            bid_role: trade.bid_role.clone().into(), // Convert MarketRole to String
            bid_fee: trade.bid_fee.to_string(),
        }
    }
}
pub fn convert_trades(trades: Vec<Trade>) -> Vec<ProtoTrade> {
    trades.iter().map(ProtoTrade::from).collect()
}
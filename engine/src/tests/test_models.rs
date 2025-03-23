use std::{str::FromStr, sync::Arc};

use bigdecimal::BigDecimal;
use common::utils::{self, get_uuid_string};
use database::models::models::{OrderStatus, TimeInForce};

use crate::models::trade_order::{OrderSide, OrderType, TradeOrder};

pub fn create_order(
    side: OrderSide,
    price: &str,
    base_amount: &str,
    quote_amount: &str,
    order_type: OrderType,
    market_id: &str,
) -> TradeOrder {
    let market_id = if market_id.is_empty() {
        "test".to_string()
    } else {
        market_id.to_string()
    };

    TradeOrder {
        id: get_uuid_string(),
        market_id,
        order_type,
        side,
        user_id: "1".to_string(),
        price: BigDecimal::from_str(price).unwrap(),
        base_amount: BigDecimal::from_str(base_amount).unwrap(),
        quote_amount: BigDecimal::from_str(quote_amount).unwrap(),
        maker_fee: BigDecimal::from(0),
        taker_fee: BigDecimal::from(0),
        create_time: utils::get_utc_now_millis(),
        remained_base: BigDecimal::from_str(base_amount).unwrap(),
        remained_quote: BigDecimal::from_str(quote_amount).unwrap(),
        filled_base: BigDecimal::from(0),
        filled_quote: BigDecimal::from(0),
        filled_fee: BigDecimal::from(0),
        update_time: utils::get_utc_now_millis(),
        client_order_id: None,
        expires_at: Some(utils::get_utc_now_millis() + 1000 * 60 * 60 * 24),
        post_only: Some(false),
        time_in_force: Some(TimeInForce::GTC),
        status: OrderStatus::Open,
    }
}

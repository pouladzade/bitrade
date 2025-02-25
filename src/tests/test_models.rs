use std::str::FromStr;

use chrono::Utc;
use rust_decimal::Decimal;

use crate::models::order::{Order, OrderSide, OrderType};

pub fn create_order(
    side: OrderSide,
    price: &str,
    amount: &str,
    order_type: OrderType,
    market_id: &str,
) -> Order {
    let market_id = if market_id.is_empty() {
        "test".to_string()
    } else {
        market_id.to_string()
    };

    Order {
        id: uuid::Uuid::new_v4().to_string(),
        base_asset: "BTC".into(),
        quote_asset: "USD".into(),
        market_id,
        order_type,
        side,
        user_id: "1".to_string(),
        price: Decimal::from_str(price).unwrap(),
        amount: Decimal::from_str(amount).unwrap(),
        maker_fee: Decimal::ZERO,
        taker_fee: Decimal::ZERO,
        create_time: Utc::now().timestamp_millis(),
        remain: Decimal::from_str(amount).unwrap(),
        frozen: Decimal::ZERO,
        filled_base: Decimal::ZERO,
        filled_quote: Decimal::ZERO,
        filled_fee: Decimal::ZERO,
        update_time: Utc::now().timestamp_millis(),
        partially_filled: true,
    }
}

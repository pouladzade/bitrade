use std::str::FromStr;

use bigdecimal::BigDecimal;
use chrono::Utc;

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
        price: BigDecimal::from_str(price).unwrap(),
        amount: BigDecimal::from_str(amount).unwrap(),
        maker_fee: BigDecimal::from(0),
        taker_fee: BigDecimal::from(0),
        create_time: Utc::now().timestamp_millis(),
        remain: BigDecimal::from_str(amount).unwrap(),
        frozen: BigDecimal::from(0),
        filled_base: BigDecimal::from(0),
        filled_quote: BigDecimal::from(0),
        filled_fee: BigDecimal::from(0),
        update_time: Utc::now().timestamp_millis(),
        partially_filled: true,
    }
}

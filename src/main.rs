mod engine;
mod models;

use crate::engine::matching::MatchingEngine;
use crate::models::order::{Order, OrderSide, OrderType};
use tokio::runtime::Runtime;

fn main() {
    let rt = Runtime::new().unwrap();
    rt.block_on(async {
        let engine = MatchingEngine::new();
        engine.run().await;

        // Example: Submit an order
        let order = Order {
            id: 1,
            market: "BTC-USD".into(),
            order_type: OrderType::Limit,
            side: OrderSide::Buy,

            post_only: false,
            price: 50000.into(),
            amount: 1.into(),

            create_time: 0.0,
            remain: 1.into(),
            frozen: 50000.into(),

            update_time: 0.0,
            base_asset: todo!(),
            quote_asset: todo!(),
            user_id: todo!(),
            filled_base: todo!(),
            filled_quote: todo!(),
            filled_fee: todo!(),
            partially_filled: todo!(),
            maker_fee: todo!(),
            taker_fee: todo!(),
        };

        engine.submit_order(order);
    });
}

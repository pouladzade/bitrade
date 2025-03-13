use crate::models::trade_order::{OrderSide, OrderType, TradeOrder};
use crate::order_book::OrderBook;
use bigdecimal::BigDecimal;
use common::utils;
use database::persistence::Persistence;

impl<P: Persistence> OrderBook<P> {
    pub fn handle_market_depth(&mut self, order: &TradeOrder) {
        if order.order_type == OrderType::Market {
            return;
        }
        match order.side {
            OrderSide::Buy => {
                let depth = self
                    .bid_depth
                    .entry(order.price.clone())
                    .or_insert(BigDecimal::from(0));
                *depth += order.remained_base.clone();
                if utils::is_zero(depth) {
                    self.bid_depth.remove(&order.price);
                }
            }
            OrderSide::Sell => {
                let depth = self
                    .ask_depth
                    .entry(order.price.clone())
                    .or_insert(BigDecimal::from(0));
                *depth += order.remained_base.clone();
                if utils::is_zero(depth) {
                    self.ask_depth.remove(&order.price);
                }
            }
        }
    }
}

use crate::models::matched_trade::MatchedTrade;
use crate::models::trade_order::{OrderSide, OrderType, TradeOrder};
use anyhow::Result;
use bigdecimal::BigDecimal;
use common::utils;
use database::persistence::Persistence;
use std::collections::{BinaryHeap, HashMap};
use std::sync::Arc;

use super::OrderBook;

impl<P: Persistence> OrderBook<P> {
    /// Add a new order asynchronously
    pub fn new(
        persister: Arc<P>,
        base_asset: String,
        market_id: String,
        quote_asset: String,
    ) -> Self {
        let mut order_book = OrderBook {
            bids: BinaryHeap::new(),
            asks: BinaryHeap::new(),
            bid_depth: HashMap::new(),
            ask_depth: HashMap::new(),
            base_asset,
            quote_asset,
            market_id,
            persister,
            market_price: None,
        };

        order_book.recover_orders_from_db().unwrap();
        order_book
    }

    pub fn recover_orders_from_db(&mut self) -> Result<()> {
        let orders = self.persister.get_active_orders(&self.market_id)?;
        let orders_len = orders.len();

        // Clear existing depth data
        self.bid_depth.clear();
        self.ask_depth.clear();

        for order in orders {
            let trade_order: TradeOrder = order.try_into()?;
            if trade_order.order_type == OrderType::Limit {
                self.match_limit_order(trade_order)?;
            } else {
                self.cancel_order(trade_order.id)?;
            }
        }
        println!("Loaded {} orders from database", orders_len);
        Ok(())
    }

    pub fn add_order(&mut self, order: TradeOrder) -> anyhow::Result<Vec<MatchedTrade>> {
        // Validate order based on price, amount and quote_amount
        if order.order_type == OrderType::Limit && order.price <= BigDecimal::from(0) {
            return Err(anyhow::anyhow!(
                "Price must be greater than 0 for limit orders"
            ));
        }

        match order.side {
            OrderSide::Buy => {
                if order.quote_amount <= BigDecimal::from(0) {
                    return Err(anyhow::anyhow!("Quote amount must be greater than 0"));
                }
            }
            OrderSide::Sell => {
                if order.base_amount <= BigDecimal::from(0) {
                    return Err(anyhow::anyhow!("Amount must be greater than 0"));
                }
            }
        }

        Self::print_order(&order);
        println!("persist_create_order");
        self.persist_create_order(&order)?;
        println!("match_order: {:?}", order);
        if order.order_type == OrderType::Limit {
            self.match_limit_order(order)
        } else {
            self.match_market_order(order)
        }
    }

    pub fn cancel_order(&mut self, order_id: String) -> anyhow::Result<bool> {
        self.persister.cancel_order(&order_id)?;

        // Find and update bid depth if needed
        if let Some(index) = self.bids.iter().position(|o| o.id == order_id) {
            let order = self.bids.iter().nth(index).unwrap().clone();
            self.handle_market_depth(&order);
            return Ok(true);
        }

        // Find and update ask depth if needed
        if let Some(index) = self.asks.iter().position(|o| o.id == order_id) {
            let order = self.asks.iter().nth(index).unwrap().clone();
            self.handle_market_depth(&order);
            return Ok(true);
        }

        Ok(false)
    }

    pub fn get_order_by_id(&self, order_id: String) -> anyhow::Result<TradeOrder> {
        if let Some(order) = self.bids.iter().find(|o| o.id == order_id) {
            return Ok(order.clone());
        } else if let Some(order) = self.asks.iter().find(|o| o.id == order_id) {
            return Ok(order.clone());
        }
        Err(anyhow::anyhow!("can not find the order!"))
    }

    pub fn cancel_all_orders(&mut self) -> anyhow::Result<bool> {
        self.persister.cancel_all_orders(&self.market_id)?;
        self.bids.clear();
        self.asks.clear();
        self.bid_depth.clear();
        self.ask_depth.clear();
        Ok(true)
    }
}

// #[cfg(test)]
// mod tests {
//     use super::*;
//     use crate::{
//         models::trade_order::{OrderSide, OrderType, TradeOrder},
//         tests::test_models::create_persistence_mock,
//     };
//     use anyhow::Context;
//     use bigdecimal::BigDecimal;
//     use database::mock::mock_thread_safe_persistence::MockThreadSafePersistence;
//     use env_logger;
//     use std::str::FromStr;

//     fn create_order(
//         id: &str,
//         side: OrderSide,
//         price: &str,
//         base_amount: &str,
//         quote_amount: &str,
//         create_time: i64,
//         order_type: OrderType,
//     ) -> TradeOrder {
//         TradeOrder {
//             id: id.to_string(),
//             market_id: "BTC-USD".into(),
//             order_type,
//             side,
//             user_id: "1".to_string(),
//             price: BigDecimal::from_str(price).unwrap(),
//             base_amount: BigDecimal::from_str(base_amount).unwrap(),
//             quote_amount: BigDecimal::from_str(quote_amount).unwrap(),
//             maker_fee: BigDecimal::from(0),
//             taker_fee: BigDecimal::from(0),
//             create_time,
//             remained_base: BigDecimal::from_str(base_amount).unwrap(),
//             remained_quote: BigDecimal::from_str(quote_amount).unwrap(),
//             filled_base: BigDecimal::from(0),
//             filled_quote: BigDecimal::from(0),
//             filled_fee: BigDecimal::from(0),
//             update_time: create_time,
//         }
//     }

// #[test]
// fn test_order_book_matching() {
//     env_logger::init();
//     let persister = MockThreadSafePersistence::new();
//     let mut order_book = OrderBook::new(create_persistence_mock());

//     // Add a bid (buy order)
//     let bid = create_order("1", OrderSide::Buy, "50000", "1", 1, OrderType::Limit);
//     let trades = order_book
//         .add_order(bid)
//         .context("can't add order")
//         .unwrap();
//     assert_eq!(trades.len(), 0); // No trades yet

//     // Add an ask (sell order) that matches the bid
//     let ask = create_order("2", OrderSide::Sell, "50000", "1", 2, OrderType::Limit);
//     let trades = order_book
//         .add_order(ask)
//         .context("can't add order")
//         .unwrap();
//     assert_eq!(trades.len(), 1); // One trade should occur
//     println!("{:?}", trades);
//     // Verify the trade details
//     let trade = &trades[0];
//     assert_eq!(trade.price, BigDecimal::from_str("50000").unwrap());
//     assert_eq!(trade.amount, BigDecimal::from_str("1").unwrap());
//     assert_eq!(trade.taker_order_id, "2");
//     assert_eq!(trade.maker_order_id, "1");

//     // Verify the order book is empty after the match
//     assert!(order_book.bids.is_empty());
//     assert!(order_book.asks.is_empty());
// }

// #[test]
// fn test_partial_match() {
//     let mut order_book = OrderBook::new(create_persistence_mock());

//     // Add a bid (buy order)
//     let bid = create_order("1", OrderSide::Buy, "50000", "2", 1, OrderType::Limit);
//     let trades = order_book.add_order(bid).unwrap();
//     assert_eq!(trades.len(), 0); // No trades yet

//     // Add an ask (sell order) that partially matches the bid
//     let ask = create_order("2", OrderSide::Sell, "50000", "1", 2, OrderType::Limit);
//     let trades = order_book.add_order(ask).unwrap();
//     assert_eq!(trades.len(), 1); // One trade should occur
//     println!("{:?}", trades);
//     // Verify the trade details
//     let trade = &trades[0];
//     assert_eq!(trade.price, BigDecimal::from_str("50000").unwrap());
//     assert_eq!(trade.amount, BigDecimal::from_str("1").unwrap());

//     // Verify the remaining bid in the order book
//     // assert_eq!(order_book.asks.len(), 1);
//     assert_eq!(order_book.bids.len(), 1);
//     let remaining_bid = order_book.bids.peek().unwrap();
//     println!("{:?}", remaining_bid);
//     assert_eq!(remaining_bid.id, "1");
//     assert_eq!(remaining_bid.remain, BigDecimal::from_str("1").unwrap());

//     // Verify the ask is fully filled and removed
//     assert!(order_book.asks.is_empty());
// }

// #[test]
// fn test_cancel_order() {
//     let mut order_book = OrderBook::new(create_persistence_mock());

//     // Add a bid (buy order)
//     let bid = create_order("1", OrderSide::Buy, "50000", "1", 1, OrderType::Limit);
//     order_book.add_order(bid);

//     // Cancel the bid
//     let canceled = order_book.cancel_order("1".to_string()).unwrap();
//     assert_eq!(canceled, true);

//     // Verify the bid is removed from the order book
//     assert!(order_book.bids.is_empty());
// }
// }

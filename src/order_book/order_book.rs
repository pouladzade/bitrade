use crate::models::matched_trade::MatchedTrade;
use crate::models::trade_order::{determine_order_status, OrderSide, OrderType, TradeOrder};
use crate::utils::{self, generate_uuid_id, is_zero};
use anyhow::Result;
use bigdecimal::BigDecimal;
use colored::*;
use database::models::models::NewOrder;
use database::persistence::persistence::Persistence;
use std::collections::BinaryHeap;
use std::sync::Arc;

/// Order Book implementation using Binary Heaps (Priority Queues) and a Hash Map.
///
/// This structure uses a combination of Binary Heaps for the bids and asks,
/// and a Hash Map for fast order cancellation. This design strikes a good
/// balance between performance and simplicity.
///
/// - **Bids (Buy Orders):** Stored in a max-heap (highest price comes first).
/// - **Asks (Sell Orders):** Stored in a min-heap (lowest price comes first).
/// - **Order Lookup:** A Hash Map for fast lookups, mapping order ID to order.
///
/// Heaps provide:
/// - O(1) access to the best bid (highest price) and ask (lowest price).
/// - O(log n) time complexity for insertions and deletions.
///
/// The Hash Map provides:
/// - O(1) lookups for order cancellation.
///
/// This approach is efficient and scales well for most in-memory systems.
/// Binary Heaps are relatively easy to implement and understand, especially
/// in Rust, which offers a strong type system and guarantees memory safety.
///
/// Note: This implementation is simplified and may not cover all edge cases
/// and optimizations required for a production-grade exchange.
///
#[derive(Debug, Clone)]
pub struct OrderBook<P>
where
    P: Persistence + 'static,
{
    bids: BinaryHeap<TradeOrder>, // Max-heap for bids (buy orders)
    asks: BinaryHeap<TradeOrder>, // Min-heap for asks (sell orders)
    persister: Arc<P>,
    market_price: BigDecimal,
}

impl<P: Persistence> OrderBook<P> {
    /// Add a new order asynchronously
    pub fn new(persister: Arc<P>) -> Self {
        OrderBook {
            bids: BinaryHeap::new(),
            asks: BinaryHeap::new(),
            persister,
            //TODO: Get the market price from the database OR from the market
            market_price: BigDecimal::try_from(10000).unwrap(),
        }
    }

    pub fn add_order(
        &mut self,
        mut order: TradeOrder,
    ) -> anyhow::Result<Vec<MatchedTrade>, anyhow::Error> {
        let mut trades = Vec::new();

        Self::print_order(&order);
        self.persist_order(&order)?;
        match order.side {
            OrderSide::Buy => {
                // Try to match the buy order with existing sell orders (asks)
                while let Some(mut ask) = self.asks.pop() {
                    // Stop if the ask price is higher than the buy order price for Limit orders
                    if order.order_type == OrderType::Limit
                        && ask.order_type == OrderType::Limit
                        && ask.price > order.price
                    {
                        print!("{} > {}", ask.price, order.price);
                        // No more matching asks
                        self.asks.push(ask); // Push it back to the heap
                        break;
                    }

                    // Calculate the trade amount
                    let trade_amount = order.remain.clone().min(ask.remain.clone());

                    // Execute the trade
                    let trade = self.execute_trade(&mut order, &mut ask, trade_amount)?;
                    trades.push(trade);

                    // Remove the ask order if fully filled
                    if ask.remain != BigDecimal::from(0) {
                        self.asks.push(ask); // Push the modified ask back into the heap
                    }

                    // Stop if the buy order is fully filled
                    if is_zero(&order.remain) {
                        break;
                    }
                }

                // Add the remaining buy order to the order book
                if !is_zero(&order.remain) {
                    self.bids.push(order.clone());
                }
            }
            OrderSide::Sell => {
                // Try to match the sell order with existing buy orders (bids)
                while let Some(mut bid) = self.bids.pop() {
                    // Stop if the bid price is lower than the sell order price for Limit orders
                    if order.order_type == OrderType::Limit
                        && bid.order_type == OrderType::Limit
                        && bid.price < order.price
                    {
                        print!("{} < {}", bid.price, order.price);
                        // No more matching bids
                        self.bids.push(bid); // Push it back to the heap
                        break;
                    }

                    // Calculate the trade amount
                    let trade_amount = order.remain.clone().min(bid.remain.clone());
                    // Execute the trade
                    let trade = self.execute_trade(&mut order, &mut bid, trade_amount)?;
                    trades.push(trade);

                    if !is_zero(&bid.remain) {
                        self.bids.push(bid); // Push the modified bid back into the heap
                    }

                    // Stop if the sell order is fully filled
                    if is_zero(&order.remain) {
                        break;
                    }
                }

                // Add the remaining sell order to the order book
                if !is_zero(&order.remain) {
                    self.asks.push(order.clone());
                }
            }
        }
        self.print_order_book();
        Ok(trades)
    }

    /// Cancel an order by its ID.
    pub fn cancel_order(&mut self, order_id: String) -> anyhow::Result<bool, anyhow::Error> {
        let bids_initial_len = self.bids.len();
        self.bids.retain(|o| o.id != order_id);
        if self.bids.len() != bids_initial_len {
            return Ok(true);
        }
        let asks_initial_len = self.asks.len();
        self.asks.retain(|o| o.id != order_id);
        if self.asks.len() != asks_initial_len {
            return Ok(true);
        }
        return Ok(false);
    }
    pub fn get_order_by_id(
        &self,
        order_id: String,
    ) -> anyhow::Result<Option<TradeOrder>, anyhow::Error> {
        if let Some(order) = self.bids.iter().find(|o| o.id == order_id) {
            return Ok(Some(order.clone()));
        } else if let Some(order) = self.asks.iter().find(|o| o.id == order_id) {
            return Ok(Some(order.clone()));
        }
        Ok(None)
    }

    pub fn cancel_all_orders(&mut self) -> anyhow::Result<bool, anyhow::Error> {
        self.bids.clear();
        self.asks.clear();
        Ok(true)
    }

    fn execute_trade(
        &mut self,
        taker: &mut TradeOrder,
        maker: &mut TradeOrder,
        amount: BigDecimal,
    ) -> anyhow::Result<MatchedTrade, anyhow::Error> {
        let trade_id = generate_uuid_id().to_string();
        self.market_price = self.calculate_trade_price(taker, maker)?;
        let timestamp = utils::get_utc_now_time_millisecond();

        // Determine fees based on order type (market or limit)
        let maker_fee = if maker.order_type == OrderType::Market {
            maker.taker_fee.clone()
        } else {
            maker.maker_fee.clone()
        } * amount.clone();

        let taker_fee = if taker.order_type == OrderType::Market {
            taker.taker_fee.clone()
        } else {
            taker.maker_fee.clone()
        } * amount.clone();

        // Update remaining amounts after deducting fees
        taker.remain -= amount.clone() - taker_fee.clone();
        maker.remain -= amount.clone() - maker_fee.clone();

        let quote_amount = amount.clone() * self.market_price.clone();

        // Construct the trade object
        let trade = MatchedTrade {
            id: trade_id,
            timestamp,
            market_id: taker.market_id.clone(),
            price: self.market_price.clone(),
            amount: amount,
            quote_amount,
            maker_user_id: maker.user_id.clone(),
            maker_order_id: maker.id.clone(),
            maker_fee,
            taker_user_id: taker.user_id.clone(),
            taker_order_id: taker.id.clone(),

            taker_fee,
        };

        // Log trade execution
        Self::print_trade(&trade);
        self.persist_update_order(&taker)?;
        self.persist_update_order(&maker)?;
        Ok(trade)
    }

    fn calculate_trade_price(
        &self,
        taker: &TradeOrder,
        maker: &TradeOrder,
    ) -> anyhow::Result<BigDecimal, anyhow::Error> {
        match (taker.order_type, maker.order_type) {
            // Market orders trade at the market price
            (OrderType::Market, OrderType::Market) => Ok(self.market_price.clone()),

            // Market order takes the price of the existing Limit order
            (OrderType::Market, OrderType::Limit) => Ok(maker.price.clone()),
            (OrderType::Limit, OrderType::Market) => Ok(taker.price.clone()),

            // Limit orders always execute at the maker's price
            (OrderType::Limit, OrderType::Limit) => Ok(maker.price.clone()),
        }
    }

    fn persist_order(&self, order: &TradeOrder) -> Result<()> {
        let new_order: NewOrder = order.clone().into(); // Convert TradeOrder to NewOrder

        self.persister.create_order(new_order)?;
        Ok(())
    }

    fn persist_update_order(&self, order: &TradeOrder) -> Result<()> {
        self.persister.update_order(
            &order.id,
            order.remain.clone(),
            order.filled_base.clone(),
            order.filled_quote.clone(),
            order.filled_fee.clone(),
            determine_order_status(&order).as_str(),
        )?;
        Ok(())
    }
    pub fn print_bids(&self) {
        let bids_sorted: Vec<TradeOrder> = self.bids.clone().into_sorted_vec();
        let bids_reversed: Vec<TradeOrder> = bids_sorted.into_iter().rev().collect();
        for bid in bids_reversed {
            let price = match bid.order_type {
                OrderType::Market => "Market".to_string(),
                _ => bid.price.to_string(),
            };
            println!(
                "{} {} , {} {} , {} {} , {} {} , {} {}",
                "id:".green(),
                bid.id,
                "price:".green(),
                price,
                "amount:".green(),
                bid.amount,
                "remain:".green(),
                bid.remain,
                "Type:".blue(),
                String::from(bid.order_type)
            );
        }
    }

    pub fn print_asks(&self) {
        let asks_sorted: Vec<TradeOrder> = self.asks.clone().into_sorted_vec();
        let asks_reversed: Vec<TradeOrder> = asks_sorted.into_iter().rev().collect();
        for ask in asks_reversed {
            let price = match ask.order_type {
                OrderType::Market => "Market".to_string(),
                _ => ask.price.to_string(),
            };

            println!(
                "{} {} , {} {} , {} {} , {} {} , {} {}",
                "id:".red(),
                ask.id,
                "price:".red(),
                price,
                "amount:".red(),
                ask.amount,
                "remain:".red(),
                ask.remain,
                "Type:".blue(),
                String::from(ask.order_type)
            );
        }
    }

    fn print_order_book(&self) {
        println!("\n{}", "Order Book:".bold().white());
        println!("{}", "Bids (Buy Orders):".green().bold());
        self.print_bids();
        println!("{}", "Asks (Sell Orders):".red().bold());
        self.print_asks();
    }

    fn print_order(order: &TradeOrder) {
        println!(
            "\nNew Order Arrived {} {} , {} {} , {} {}, {} {}",
            "Order id:".blue(),
            order.id,
            "price:".blue(),
            order.price,
            "amount:".blue(),
            order.amount,
            "Type:".blue(),
            String::from(order.order_type)
        );
    }

    fn print_trade(trade: &MatchedTrade) {
        println!(
            "\nNew Trade Matched {} {} , {} {} , {} {} , {} {}",
            "Trade id:".cyan(),
            trade.id,
            "price:".cyan(),
            trade.price,
            "amount:".cyan(),
            trade.amount,
            "quote_amount:".cyan(),
            trade.quote_amount
        );
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::{
        models::trade_order::{OrderSide, OrderType, TradeOrder},
        tests::test_models::create_persistence_mock,
    };
    use anyhow::Context;
    use bigdecimal::BigDecimal;
    use database::mock::mock_thread_safe_persistence::MockThreadSafePersistence;
    use env_logger;
    use std::str::FromStr;

    fn create_order(
        id: &str,
        side: OrderSide,
        price: &str,
        amount: &str,
        create_time: i64,
        order_type: OrderType,
    ) -> TradeOrder {
        TradeOrder {
            id: id.to_string(),
            market_id: "BTC-USD".into(),
            order_type,
            side,
            user_id: "1".to_string(),
            price: BigDecimal::from_str(price).unwrap(),
            amount: BigDecimal::from_str(amount).unwrap(),
            maker_fee: BigDecimal::from(0),
            taker_fee: BigDecimal::from(0),
            create_time,
            remain: BigDecimal::from_str(amount).unwrap(),

            filled_base: BigDecimal::from(0),
            filled_quote: BigDecimal::from(0),
            filled_fee: BigDecimal::from(0),
            update_time: create_time,
        }
    }

    #[test]
    fn test_order_book_matching() {
        env_logger::init();
        let persister = MockThreadSafePersistence::new();
        let mut order_book = OrderBook::new(create_persistence_mock());

        // Add a bid (buy order)
        let bid = create_order("1", OrderSide::Buy, "50000", "1", 1, OrderType::Limit);
        let trades = order_book
            .add_order(bid)
            .context("can't add order")
            .unwrap();
        assert_eq!(trades.len(), 0); // No trades yet

        // Add an ask (sell order) that matches the bid
        let ask = create_order("2", OrderSide::Sell, "50000", "1", 2, OrderType::Limit);
        let trades = order_book
            .add_order(ask)
            .context("can't add order")
            .unwrap();
        assert_eq!(trades.len(), 1); // One trade should occur
        println!("{:?}", trades);
        // Verify the trade details
        let trade = &trades[0];
        assert_eq!(trade.price, BigDecimal::from_str("50000").unwrap());
        assert_eq!(trade.amount, BigDecimal::from_str("1").unwrap());
        assert_eq!(trade.taker_order_id, "2");
        assert_eq!(trade.maker_order_id, "1");

        // Verify the order book is empty after the match
        assert!(order_book.bids.is_empty());
        assert!(order_book.asks.is_empty());
    }

    #[test]
    fn test_partial_match() {
        let mut order_book = OrderBook::new(create_persistence_mock());

        // Add a bid (buy order)
        let bid = create_order("1", OrderSide::Buy, "50000", "2", 1, OrderType::Limit);
        let trades = order_book.add_order(bid).unwrap();
        assert_eq!(trades.len(), 0); // No trades yet

        // Add an ask (sell order) that partially matches the bid
        let ask = create_order("2", OrderSide::Sell, "50000", "1", 2, OrderType::Limit);
        let trades = order_book.add_order(ask).unwrap();
        assert_eq!(trades.len(), 1); // One trade should occur
        println!("{:?}", trades);
        // Verify the trade details
        let trade = &trades[0];
        assert_eq!(trade.price, BigDecimal::from_str("50000").unwrap());
        assert_eq!(trade.amount, BigDecimal::from_str("1").unwrap());

        // Verify the remaining bid in the order book
        // assert_eq!(order_book.asks.len(), 1);
        assert_eq!(order_book.bids.len(), 1);
        let remaining_bid = order_book.bids.peek().unwrap();
        println!("{:?}", remaining_bid);
        assert_eq!(remaining_bid.id, "1");
        assert_eq!(remaining_bid.remain, BigDecimal::from_str("1").unwrap());

        // Verify the ask is fully filled and removed
        assert!(order_book.asks.is_empty());
    }

    #[test]
    fn test_cancel_order() {
        let mut order_book = OrderBook::new(create_persistence_mock());

        // Add a bid (buy order)
        let bid = create_order("1", OrderSide::Buy, "50000", "1", 1, OrderType::Limit);
        order_book.add_order(bid);

        // Cancel the bid
        let canceled = order_book.cancel_order("1".to_string()).unwrap();
        assert_eq!(canceled, true);

        // Verify the bid is removed from the order book
        assert!(order_book.bids.is_empty());
    }
}

use crate::models::order::{Order, OrderSide, OrderType};
use crate::models::trade::{MarketRole, Trade};
use rust_decimal::Decimal;
use std::collections::BinaryHeap;
use std::collections::HashMap;

use colored::*;
use std::time::{SystemTime, UNIX_EPOCH}; // Import the crate
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
pub(crate) trait OrderBookTrait {
    fn new() -> Self;
    fn add_order(&mut self, order: Order) -> Vec<Trade>;
    fn cancel_order(&mut self, order_id: u64) -> bool;
    fn get_bids(&self) -> BinaryHeap<Order>;
    fn get_asks(&self) -> BinaryHeap<Order>;
    fn get_orders(&self) -> HashMap<u64, Order>;
    fn get_order_count_by_side(&self, side: OrderSide) -> usize;
    fn get_order_by_id(&self, order_id: u64) -> Option<Order>;
    fn get_order_by_user(&self, user_id: u32) -> Vec<Order>;
    fn cancel_all_orders(&mut self) -> bool;
}

#[derive(Debug, Clone)]
pub struct OrderBook {
    bids: BinaryHeap<Order>,     // Max-heap for bids (buy orders)
    asks: BinaryHeap<Order>,     // Min-heap for asks (sell orders)
    orders: HashMap<u64, Order>, // Order ID to Order mapping
}

impl OrderBookTrait for OrderBook {
    /// Add a new order asynchronously
    fn new() -> Self {
        OrderBook {
            bids: BinaryHeap::new(),
            asks: BinaryHeap::new(),
            orders: HashMap::new(),
        }
    }

    fn add_order(&mut self, mut order: Order) -> Vec<Trade> {
        let mut trades = Vec::new();
        Self::print_order(&order);
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
                    let trade_amount = order.remain.min(ask.remain);
                    let trade_price = ask.price;

                    // Execute the trade
                    let trade =
                        Self::execute_trade(&mut order, &mut ask, trade_amount, trade_price);
                    trades.push(trade);

                    // Remove the ask order if fully filled
                    if ask.remain.is_zero() {
                        self.orders.remove(&ask.id);
                    } else {
                        self.asks.push(ask); // Push the modified ask back into the heap
                    }

                    // Stop if the buy order is fully filled
                    if order.remain.is_zero() {
                        break;
                    }
                }

                // Add the remaining buy order to the order book
                if !order.remain.is_zero() {
                    self.bids.push(order.clone());
                    self.orders.insert(order.id, order);
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
                    let trade_amount = order.remain.min(bid.remain);
                    let trade_price = bid.price;

                    // Execute the trade
                    let trade =
                        Self::execute_trade(&mut bid, &mut order, trade_amount, trade_price);
                    trades.push(trade);

                    // Remove the bid order if fully filled
                    if bid.remain.is_zero() {
                        self.orders.remove(&bid.id);
                    } else {
                        self.bids.push(bid); // Push the modified bid back into the heap
                    }

                    // Stop if the sell order is fully filled
                    if order.remain.is_zero() {
                        break;
                    }
                }

                // Add the remaining sell order to the order book
                if !order.remain.is_zero() {
                    self.asks.push(order.clone());
                    self.orders.insert(order.id, order);
                }
            }
        }
        self.print_order_book();
        trades
    }

    /// Cancel an order by its ID.
    fn cancel_order(&mut self, order_id: u64) -> bool {
        if let Some(order) = self.orders.remove(&order_id) {
            match order.side {
                OrderSide::Buy => self.bids.retain(|o| o.id != order_id),
                OrderSide::Sell => self.asks.retain(|o| o.id != order_id),
            }
            true
        } else {
            false
        }
    }
    fn get_bids(&self) -> BinaryHeap<Order> {
        self.bids.clone()
    }
    fn get_asks(&self) -> BinaryHeap<Order> {
        self.asks.clone()
    }
    fn get_orders(&self) -> HashMap<u64, Order> {
        self.orders.clone()
    }
    fn get_order_count_by_side(&self, side: OrderSide) -> usize {
        match side {
            OrderSide::Buy => self.bids.len(),
            OrderSide::Sell => self.asks.len(),
        }
    }
    fn get_order_by_id(&self, order_id: u64) -> Option<Order> {
        self.orders.get(&order_id).cloned()
    }

    fn get_order_by_user(&self, user_id: u32) -> Vec<Order> {
        self.orders
            .values()
            .filter(|o| o.user_id == user_id)
            .cloned()
            .collect()
    }

    fn cancel_all_orders(&mut self) -> bool {
        self.bids.clear();
        self.asks.clear();
        self.orders.clear();
        true
    }
}

impl OrderBook {
    fn execute_trade(bid: &mut Order, ask: &mut Order, amount: Decimal, price: Decimal) -> Trade {
        let trade_id = OrderBook::generate_trade_id();
        let ask_fee = if ask.order_type == OrderType::Market {
            ask.taker_fee
        } else {
            ask.maker_fee
        } * amount;
        let bid_fee = if bid.order_type == OrderType::Market {
            bid.taker_fee
        } else {
            bid.maker_fee
        } * amount;

        bid.remain -= amount - bid_fee;
        ask.remain -= amount - ask_fee;
        let quote_amount = amount * price;
        Self::print_trade(&Trade {
            id: trade_id,
            timestamp: SystemTime::now()
                .duration_since(UNIX_EPOCH)
                .unwrap()
                .as_secs_f64(),
            market: bid.market.clone(),
            base_asset: bid.base_asset.clone(),
            quote_asset: bid.quote_asset.clone(),
            price,
            amount,
            quote_amount,
            ask_user_id: ask.user_id,
            ask_order_id: ask.id,
            ask_role: MarketRole::Maker,
            ask_fee,
            bid_user_id: bid.user_id,
            bid_order_id: bid.id,
            bid_role: MarketRole::Taker,
            bid_fee,
        });
        Trade {
            id: trade_id,
            timestamp: SystemTime::now()
                .duration_since(UNIX_EPOCH)
                .unwrap()
                .as_secs_f64(),
            market: bid.market.clone(),
            base_asset: bid.base_asset.clone(),
            quote_asset: bid.quote_asset.clone(),
            price,
            amount,
            quote_amount,
            ask_user_id: ask.user_id,
            ask_order_id: ask.id,
            ask_role: MarketRole::Maker,
            ask_fee,
            bid_user_id: bid.user_id,
            bid_order_id: bid.id,
            bid_role: MarketRole::Taker,
            bid_fee,
        }
    }

    /// Generates a unique trade ID
    fn generate_trade_id() -> u64 {
        use std::sync::atomic::{AtomicU64, Ordering};
        static TRADE_ID_COUNTER: AtomicU64 = AtomicU64::new(1);
        TRADE_ID_COUNTER.fetch_add(1, Ordering::Relaxed)
    }

    fn print_order_book(&self) {
        println!("\n{}", "Order Book:".bold().white());

        println!("{}", "Bids (Buy Orders):".green().bold());
        for bid in self.bids.iter() {
            println!(
                "{} {} , {} {} , {} {} , {} {}",
                "id:".green(),
                bid.id,
                "price:".green(),
                bid.price,
                "amount:".green(),
                bid.amount,
                "remain:".green(),
                bid.remain
            );
        }

        println!("{}", "Asks (Sell Orders):".red().bold());
        for ask in self.asks.iter() {
            println!(
                "{} {} , {} {} , {} {} , {} {}",
                "id:".red(),
                ask.id,
                "price:".red(),
                ask.price,
                "amount:".red(),
                ask.amount,
                "remain:".red(),
                ask.remain
            );
        }
    }
    fn print_order(order: &Order) {
        println!(
            "\nNew Order Arrived {} {} , {} {} , {} {}",
            "Order id:".blue(),
            order.id,
            "price:".blue(),
            order.price,
            "amount:".blue(),
            order.amount
        );
    }

    fn print_trade(trade: &Trade) {
        println!(
            "\nNew Trade Matched {} {} , {} {} , {} {} , {} {} , {} {}",
            "Trade id:".cyan(),
            trade.id,
            "price:".cyan(),
            trade.price,
            "amount:".cyan(),
            trade.amount,
            "quote_amount:".cyan(),
            trade.quote_amount,
            "quote_asset:".cyan(),
            trade.quote_asset,
        );
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::models::order::{self, Order, OrderSide, OrderType};
    use env_logger;
    use log::{debug, error, info, warn};
    use rust_decimal::Decimal;
    use std::str::FromStr;

    fn create_order(
        id: u64,
        side: OrderSide,
        price: &str,
        amount: &str,
        create_time: f64,
        order_type: OrderType,
    ) -> Order {
        Order {
            id,
            base_asset: "BTC".into(),
            quote_asset: "USD".into(),
            market: "BTC-USD".into(),
            order_type,
            side,
            user_id: 1,
            post_only: false,
            price: Decimal::from_str(price).unwrap(),
            amount: Decimal::from_str(amount).unwrap(),
            maker_fee: Decimal::ZERO,
            taker_fee: Decimal::ZERO,
            create_time,
            remain: Decimal::from_str(amount).unwrap(),
            frozen: Decimal::ZERO,
            filled_base: Decimal::ZERO,
            filled_quote: Decimal::ZERO,
            filled_fee: Decimal::ZERO,
            update_time: create_time,
            partially_filled: true,
        }
    }

    #[test]
    fn test_order_book_matching() {
        env_logger::init();
        let mut order_book = OrderBook::new();

        // Add a bid (buy order)
        let bid = create_order(1, OrderSide::Buy, "50000", "1", 1.0, OrderType::Limit);
        let trades = order_book.add_order(bid);
        assert_eq!(trades.len(), 0); // No trades yet

        // Add an ask (sell order) that matches the bid
        let ask = create_order(2, OrderSide::Sell, "50000", "1", 2.0, OrderType::Limit);
        let trades = order_book.add_order(ask);
        assert_eq!(trades.len(), 1); // One trade should occur
        println!("{:?}", trades);
        // Verify the trade details
        let trade = &trades[0];
        assert_eq!(trade.price, Decimal::from_str("50000").unwrap());
        assert_eq!(trade.amount, Decimal::from_str("1").unwrap());
        assert_eq!(trade.ask_order_id, 2);
        assert_eq!(trade.bid_order_id, 1);

        // Verify the order book is empty after the match
        assert!(order_book.bids.is_empty());
        assert!(order_book.asks.is_empty());
    }

    #[test]
    fn test_partial_match() {
        let mut order_book = OrderBook::new();

        // Add a bid (buy order)
        let bid = create_order(1, OrderSide::Buy, "50000", "2", 1.0, OrderType::Limit);
        let trades = order_book.add_order(bid);
        assert_eq!(trades.len(), 0); // No trades yet

        // Add an ask (sell order) that partially matches the bid
        let ask = create_order(2, OrderSide::Sell, "50000", "1", 2.0, OrderType::Limit);
        let trades = order_book.add_order(ask);
        assert_eq!(trades.len(), 1); // One trade should occur
        println!("{:?}", trades);
        // Verify the trade details
        let trade = &trades[0];
        assert_eq!(trade.price, Decimal::from_str("50000").unwrap());
        assert_eq!(trade.amount, Decimal::from_str("1").unwrap());

        // Verify the remaining bid in the order book
        // assert_eq!(order_book.asks.len(), 1);
        assert_eq!(order_book.bids.len(), 1);
        let remaining_bid = order_book.bids.peek().unwrap();
        println!("{:?}", remaining_bid);
        assert_eq!(remaining_bid.id, 1);
        assert_eq!(remaining_bid.remain, Decimal::from_str("1").unwrap());

        // Verify the ask is fully filled and removed
        assert!(order_book.asks.is_empty());
    }

    #[test]
    fn test_cancel_order() {
        let mut order_book = OrderBook::new();

        // Add a bid (buy order)
        let bid = create_order(1, OrderSide::Buy, "50000", "1", 1.0, OrderType::Limit);
        order_book.add_order(bid);

        // Cancel the bid
        let canceled = order_book.cancel_order(1);
        assert_eq!(canceled, true);

        // Verify the bid is removed from the order book
        assert!(order_book.bids.is_empty());
        assert!(order_book.orders.is_empty());
    }
}

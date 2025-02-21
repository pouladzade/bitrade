use crate::models::order::{Order, OrderSide};
use crate::models::trade::{MarketRole, Trade};
use rust_decimal::Decimal;
use std::collections::BinaryHeap;
use std::collections::HashMap;
use std::sync::{Arc, Mutex};
use std::time::{SystemTime, UNIX_EPOCH};

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
#[derive(Debug, Clone)]
pub struct OrderBook {
    bids: BinaryHeap<Order>,     // Max-heap for bids (buy orders)
    asks: BinaryHeap<Order>,     // Min-heap for asks (sell orders)
    orders: HashMap<u64, Order>, // Order ID to Order mapping
}

impl OrderBook {
    /// Add a new order asynchronously
    pub fn new() -> Self {
        OrderBook {
            bids: BinaryHeap::new(),
            asks: BinaryHeap::new(),
            orders: HashMap::new(),
        }
    }

    /// Add a new order and attempt to match it.
    ///
    /// This function handles both buy and sell orders. It attempts to match the
    /// incoming order against existing orders in the book. If a match is found,
    /// a trade is executed. If no match is found, the order is added to the respective
    /// side of the order book (bids or asks).
    ///
    /// # Matching Logic:
    /// - **Buy Order (OrderSide::Buy):**
    ///   - Tries to match with the lowest available ask (sell order).
    ///   - If no match is found, the order is added to the bids.
    ///
    /// - **Sell Order (OrderSide::Sell):**
    ///   - Tries to match with the highest available bid (buy order).
    ///   - If no match is found, the order is added to the asks.
    pub fn add_order(&mut self, mut order: Order) -> Vec<Trade> {
        let mut trades = Vec::new();

        match order.side {
            OrderSide::Buy => {
                // Try to match the buy order with existing sell orders (asks)
                while let Some(mut ask) = self.asks.pop() {
                    // Stop if the ask price is higher than the buy order price
                    if ask.price > order.price {
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
                    if bid.price < order.price {
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

        trades
    }

    /// Execute a trade between two orders.
    ///
    /// This function handles the execution of a trade between two orders in the order book.
    /// It updates the balances of the orders (remaining amounts), calculates the trade amount and fees, and
    /// returns a `Trade` object containing the details of the trade.
    ///
    /// ### Matching Logic:
    /// - **Maker**: The order already in the book (counter-order).
    /// - **Taker**: The incoming order that initiates the trade.
    ///
    /// ### Trade Execution:
    /// - Deducts the traded amount from both the maker's and taker's remaining balance.
    /// - Calculates the quote amount based on the price and trade amount.
    /// - Generates a unique trade ID.
    ///
    /// ### Returned `Trade` Object:
    /// Contains the details of the executed trade, including:
    /// - Trade ID
    /// - Timestamp
    /// - Market details
    /// - Base and quote assets
    /// - Trade amount, price, and quote amount
    /// - User IDs, order IDs, and roles (maker/taker)
    /// - Fees associated with each side (maker/taker)
    ///
    /// # Parameters:
    /// - `order`: The incoming order (taker).
    /// - `counter_order`: The order in the book (maker).
    /// - `amount`: The amount to be traded.
    /// - `price`: The price at which the trade is executed.
    ///
    /// # Returns:
    /// A `Trade` object with details of the executed trade.
    ///
    fn execute_trade(bid: &mut Order, ask: &mut Order, amount: Decimal, price: Decimal) -> Trade {
        bid.remain -= amount;
        ask.remain -= amount;
        let quote_amount = amount * price;

        let trade_id = OrderBook::generate_trade_id();

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
            ask_fee: ask.taker_fee * amount,
            bid_user_id: bid.user_id,
            bid_order_id: bid.id,
            bid_role: MarketRole::Taker,
            bid_fee: bid.taker_fee * amount,
            ask_order: None,
            bid_order: None,
            state_before: None,
            state_after: None,
        }
    }

    /// Generates a unique trade ID
    fn generate_trade_id() -> u64 {
        use std::sync::atomic::{AtomicU64, Ordering};
        static TRADE_ID_COUNTER: AtomicU64 = AtomicU64::new(1);
        TRADE_ID_COUNTER.fetch_add(1, Ordering::Relaxed)
    }

    /// Cancel an order by its ID.
    pub fn cancel_order(&mut self, order_id: u64) -> Option<Order> {
        if let Some(order) = self.orders.remove(&order_id) {
            match order.side {
                OrderSide::Buy => self.bids.retain(|o| o.id != order_id),
                OrderSide::Sell => self.asks.retain(|o| o.id != order_id),
            }
            Some(order)
        } else {
            None
        }
    }
}

#[derive(Debug, Clone)]
/// A thread-safe wrapper around the `OrderBook` to allow concurrent access.
///
/// The `SharedOrderBook` struct provides a safe interface to interact with the `OrderBook` in a multithreaded context.
/// It uses an `Arc<Mutex<OrderBook>>` to ensure that the `OrderBook` can be accessed safely by multiple threads.
///
/// The `add_order` and `cancel_order` methods lock the `OrderBook` for modification, ensuring that operations on the order book
/// are performed atomically, even when accessed by multiple concurrent tasks.
///
/// ### Methods:
///
/// - `new()`: Creates a new `SharedOrderBook` instance, initializing the `OrderBook` within a `Mutex` for safe access.
/// - `add_order(&self, order: Order)`: Adds a new order to the order book and attempts to match it with existing orders.
/// - `cancel_order(&self, order_id: u64)`: Cancels an order by its ID, removing it from the order book if it exists.
pub struct SharedOrderBook {
    inner: Arc<Mutex<OrderBook>>,
}

impl SharedOrderBook {
    /// Creates a new `SharedOrderBook` instance.
    ///
    /// This method initializes the inner `OrderBook` within a `Mutex`, which is wrapped in an `Arc` for safe sharing across threads.
    ///
    /// # Returns:
    /// A new `SharedOrderBook` instance.
    pub fn new() -> Self {
        SharedOrderBook {
            inner: Arc::new(Mutex::new(OrderBook::new())),
        }
    }

    /// Adds a new order to the order book and attempts to match it with existing orders.
    ///
    /// This method locks the inner `OrderBook`, calls its `add_order` method to process the new order, and returns any resulting trades.
    /// The lock ensures that only one thread can modify the order book at a time.
    ///
    /// # Parameters:
    /// - `order`: The order to add to the order book.
    ///
    /// # Returns:
    /// A vector of `Trade` objects that resulted from the matching process.
    pub async fn add_order(&self, order: Order) -> Vec<Trade> {
        let mut order_book = self.inner.lock().unwrap();
        order_book.add_order(order)
    }

    /// Cancels an existing order by its ID.
    ///
    /// This method locks the inner `OrderBook`, calls its `cancel_order` method to remove the order if it exists,
    /// and returns the canceled order if successful.
    ///
    /// # Parameters:
    /// - `order_id`: The ID of the order to cancel.
    ///
    /// # Returns:
    /// An `Option<Order>`, which will contain the canceled order if it was found, or `None` if no matching order was found.
    pub async fn cancel_order(&self, order_id: u64) -> Option<Order> {
        let mut order_book = self.inner.lock().unwrap();
        order_book.cancel_order(order_id)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::models::order::{Order, OrderSide, OrderType};
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
    ) -> Order {
        Order {
            id,
            base_asset: "BTC".into(),
            quote_asset: "USD".into(),
            market: "BTC-USD".into(),
            order_type: OrderType::Limit,
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
        let bid = create_order(1, OrderSide::Buy, "50000", "1", 1.0);
        let trades = order_book.add_order(bid);
        assert_eq!(trades.len(), 0); // No trades yet

        // Add an ask (sell order) that matches the bid
        let ask = create_order(2, OrderSide::Sell, "50000", "1", 2.0);
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
        let bid = create_order(1, OrderSide::Buy, "50000", "2", 1.0);
        let trades = order_book.add_order(bid);
        assert_eq!(trades.len(), 0); // No trades yet

        // Add an ask (sell order) that partially matches the bid
        let ask = create_order(2, OrderSide::Sell, "50000", "1", 2.0);
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
        let bid = create_order(1, OrderSide::Buy, "50000", "1", 1.0);
        order_book.add_order(bid);

        // Cancel the bid
        let canceled_order = order_book.cancel_order(1).unwrap();
        assert_eq!(canceled_order.id, 1);

        // Verify the bid is removed from the order book
        assert!(order_book.bids.is_empty());
        assert!(order_book.orders.is_empty());
    }
}

// trading_engine.rs

use std::sync::{Arc, Mutex, mpsc};
use std::thread;
use std::collections::{HashMap, BTreeMap};
use bigdecimal::BigDecimal;
use anyhow::{Result, anyhow};
use log::{debug, info, error, warn};
use uuid::Uuid;

use crate::models::{
    Balance, Market, NewOrder, NewTrade, Order, OrderSide, OrderStatus, OrderType, Trade
};
use crate::persistence::ThreadSafePersistence;

// Message types for the persistence thread pool
enum PersistenceMessage {
    SaveOrder(NewOrder),
    UpdateOrder(String, BigDecimal, BigDecimal, BigDecimal, BigDecimal, BigDecimal, bool, String),
    SaveTrade(NewTrade),
    SaveMultipleTrades(Vec<NewTrade>),
    UpdateBalance(String, String, BigDecimal, BigDecimal),
    UpdateMarketStats(String, BigDecimal, BigDecimal, BigDecimal, BigDecimal, BigDecimal),
    Shutdown,
}

// Order book entry
struct OrderBookEntry {
    order_id: String,
    user_id: String,
    price: BigDecimal,
    remain: BigDecimal,
    create_time: i64,
}

// Order book for a market
struct OrderBook {
    market_id: String,
    buy_orders: BTreeMap<BigDecimal, Vec<OrderBookEntry>>, // Descending order
    sell_orders: BTreeMap<BigDecimal, Vec<OrderBookEntry>>, // Ascending order
}

impl OrderBook {
    fn new(market_id: &str) -> Self {
        Self {
            market_id: market_id.to_string(),
            buy_orders: BTreeMap::new(),
            sell_orders: BTreeMap::new(),
        }
    }

    fn add_order(&mut self, order: &Order) -> Result<()> {
        let side = order.get_side()?;
        
        let entry = OrderBookEntry {
            order_id: order.id.clone(),
            user_id: order.user_id.clone(),
            price: order.price.clone(),
            remain: order.remain.clone(),
            create_time: order.create_time,
        };

        match side {
            OrderSide::Buy => {
                // Price descending for buy orders
                let price_key = order.price.clone();
                let orders = self.buy_orders.entry(price_key).or_insert_with(Vec::new);
                orders.push(entry);
                // Sort by time (oldest first)
                orders.sort_by_key(|e| e.create_time);
            },
            OrderSide::Sell => {
                // Price ascending for sell orders
                let price_key = order.price.clone();
                let orders = self.sell_orders.entry(price_key).or_insert_with(Vec::new);
                orders.push(entry);
                // Sort by time (oldest first)
                orders.sort_by_key(|e| e.create_time);
            }
        }

        Ok(())
    }

    fn remove_order(&mut self, order_id: &str, side: OrderSide, price: &BigDecimal) -> Result<()> {
        let orders_map = match side {
            OrderSide::Buy => &mut self.buy_orders,
            OrderSide::Sell => &mut self.sell_orders,
        };

        if let Some(orders) = orders_map.get_mut(price) {
            let position = orders.iter().position(|e| e.order_id == order_id);
            if let Some(idx) = position {
                orders.remove(idx);
                if orders.is_empty() {
                    orders_map.remove(price);
                }
            }
        }

        Ok(())
    }

    fn update_order(&mut self, order_id: &str, side: OrderSide, price: &BigDecimal, new_remain: &BigDecimal) -> Result<()> {
        let orders_map = match side {
            OrderSide::Buy => &mut self.buy_orders,
            OrderSide::Sell => &mut self.sell_orders,
        };

        if let Some(orders) = orders_map.get_mut(price) {
            for entry in orders.iter_mut() {
                if entry.order_id == order_id {
                    entry.remain = new_remain.clone();
                    break;
                }
            }
        }

        Ok(())
    }
    
    fn best_buy_price(&self) -> Option<BigDecimal> {
        self.buy_orders.keys().next().cloned()
    }
    
    fn best_sell_price(&self) -> Option<BigDecimal> {
        self.sell_orders.keys().next().cloned()
    }
}

// The Trading Engine
pub struct TradingEngine {
    // Thread-safe persistence manager
    persistence: Arc<ThreadSafePersistence>,
    
    // Order books by market ID
    order_books: Arc<Mutex<HashMap<String, OrderBook>>>,
    
    // Persistence thread pool
    thread_count: usize,
    sender: mpsc::Sender<PersistenceMessage>,
    worker_handles: Vec<thread::JoinHandle<()>>,
}

impl TradingEngine {
    pub fn new(persistence: ThreadSafePersistence, thread_count: usize) -> Self {
        let persistence = Arc::new(persistence);
        let order_books = Arc::new(Mutex::new(HashMap::new()));
        
        // Create channel for persistence messages
        let (sender, receiver) = mpsc::channel();
        let receiver = Arc::new(Mutex::new(receiver));
        
        // Spawn worker threads for persistence
        let mut worker_handles = Vec::with_capacity(thread_count);
        for i in 0..thread_count {
            let thread_persistence = Arc::clone(&persistence);
            let thread_receiver = Arc::clone(&receiver);
            
            let handle = thread::spawn(move || {
                info!("Persistence worker {} started", i);
                loop {
                    // Get message from queue
                    let message = {
                        let receiver = thread_receiver.lock().unwrap();
                        match receiver.recv() {
                            Ok(msg) => msg,
                            Err(_) => break, // Channel closed
                        }
                    };
                    
                    // Process message
                    match message {
                        PersistenceMessage::SaveOrder(new_order) => {
                            if let Err(e) = thread_persistence.create_order(new_order) {
                                error!("Failed to save order: {}", e);
                            }
                        },
                        PersistenceMessage::UpdateOrder(order_id, remain, frozen, filled_base, 
                                                       filled_quote, filled_fee, partially_filled, status) => {
                            if let Err(e) = thread_persistence.update_order(&order_id, remain, frozen, 
                                                                        filled_base, filled_quote, 
                                                                        filled_fee, partially_filled, &status) {
                                error!("Failed to update order {}: {}", order_id, e);
                            }
                        },
                        PersistenceMessage::SaveTrade(new_trade) => {
                            if let Err(e) = thread_persistence.create_trade(new_trade) {
                                error!("Failed to save trade: {}", e);
                            }
                        },
                        PersistenceMessage::SaveMultipleTrades(new_trades) => {
                            if let Err(e) = thread_persistence.create_trades(new_trades) {
                                error!("Failed to save multiple trades: {}", e);
                            }
                        },
                        PersistenceMessage::UpdateBalance(user_id, asset, available_delta, frozen_delta) => {
                            if let Err(e) = thread_persistence.update_or_create_balance(&user_id, &asset, 
                                                                                    available_delta, frozen_delta) {
                                error!("Failed to update balance for user {} asset {}: {}", user_id, asset, e);
                            }
                        },
                        PersistenceMessage::UpdateMarketStats(market_id, high_24h, low_24h, volume_24h, 
                                                            price_change_24h, last_price) => {
                            if let Err(e) = thread_persistence.update_market_stats(&market_id, high_24h, 
                                                                               low_24h, volume_24h, 
                                                                               price_change_24h, last_price) {
                                error!("Failed to update market stats for {}: {}", market_id, e);
                            }
                        },
                        PersistenceMessage::Shutdown => {
                            info!("Persistence worker {} shutting down", i);
                            break;
                        }
                    }
                }
                info!("Persistence worker {} terminated", i);
            });
            
            worker_handles.push(handle);
        }
        
        Self {
            persistence,
            order_books,
            thread_count,
            sender,
            worker_handles,
        }
    }
    
    // Initialize the engine with existing market data
    pub fn initialize(&self) -> Result<()> {
        // Load markets
        let markets = self.persistence.list_markets()?;
        
        // Initialize order books
        let mut order_books = self.order_books.lock().map_err(|_| anyhow!("Order books lock poisoned"))?;
        
        for market in &markets {
            info!("Initializing order book for market: {}", market.id);
            order_books.insert(market.id.clone(), OrderBook::new(&market.id));
        }
        
        // Load open orders into order books
        for market in &markets {
            let open_orders = self.persistence.get_open_orders_for_market(&market.id)?;
            
            if let Some(order_book) = order_books.get_mut(&market.id) {
                for order in open_orders {
                    info!("Adding order {} to order book", order.id);
                    order_book.add_order(&order)?;
                }
            }
        }
        
        info!("Trading engine initialized with {} markets", markets.len());
        Ok(())
    }
    
    // Process a new order
    pub fn process_order(&self, new_order: NewOrder) -> Result<Order> {
        // Clone necessary values for persistence
        let order_id = new_order.id.clone();
        let market_id = new_order.market_id.clone();
        let user_id = new_order.user_id.clone();
        let side = OrderSide::from_str(&new_order.side)?;
        let order_type = OrderType::from_str(&new_order.order_type)?;
        let price = new_order.price.clone();
        let amount = new_order.amount.clone();
        
        // Send to persistence thread pool
        self.sender.send(PersistenceMessage::SaveOrder(new_order))
            .map_err(|_| anyhow!("Failed to send persistence message"))?;
        
        // Get order book
        let mut order_books = self.order_books.lock().map_err(|_| anyhow!("Order books lock poisoned"))?;
        let order_book = order_books.get_mut(&market_id)
            .ok_or_else(|| anyhow!("Market not found: {}", market_id))?;
        
        // Create order object (simulating what would be returned from DB)
        let order = Order {
            id: order_id.clone(),
            market_id: market_id.clone(),
            user_id: user_id.clone(),
            order_type: order_type.as_str().to_string(),
            side: side.as_str().to_string(),
            price: price.clone(),
            amount: amount.clone(),
            maker_fee: BigDecimal::from(0), // These would come from DB
            taker_fee: BigDecimal::from(0),
            create_time: chrono::Utc::now().timestamp(),
            remain: amount.clone(),
            frozen: price.clone() * amount.clone(),
            filled_base: BigDecimal::from(0),
            filled_quote: BigDecimal::from(0),
            filled_fee: BigDecimal::from(0),
            update_time: chrono::Utc::now().timestamp(),
            partially_filled: false,
            status: OrderStatus::Open.as_str().to_string(),
        };
        
        // Match order (for limit orders)
        if order_type == OrderType::Limit {
            self.match_limit_order(order_book, &order, side, &price, &amount)?;
        } else if order_type == OrderType::Market {
            self.match_market_order(order_book, &order, side, &amount)?;
        }
        
        // Add remaining order to book if it's a limit order and wasn't fully filled
        if order_type == OrderType::Limit {
            // Reload the order after potential matching to get current state
            if let Some(updated_order) = self.persistence.get_order(&order_id)? {
                if updated_order.status == OrderStatus::Open.as_str() {
                    order_book.add_order(&updated_order)?;
                }
            }
        }
        
        // Return the created order
        Ok(order)
    }
    
    // Match a limit order against the order book
    fn match_limit_order(&self, order_book: &mut OrderBook, order: &Order, 
                        side: OrderSide, price: &BigDecimal, amount: &BigDecimal) -> Result<()> {
        match side {
            OrderSide::Buy => {
                // Match against sell orders with price <= buy price
                let mut matched_prices = Vec::new();
                let mut to_match = amount.clone();
                
                // Find all eligible sell prices
                for (&sell_price, _) in order_book.sell_orders.iter() {
                    if &sell_price <= price {
                        matched_prices.push(sell_price.clone());
                    } else {
                        break; // Prices are sorted, so we can break early
                    }
                }
                
                // Match against each price level
                for matched_price in matched_prices {
                    if to_match <= BigDecimal::from(0) {
                        break;
                    }
                    
                    if let Some(sell_orders) = order_book.sell_orders.get_mut(&matched_price) {
                        // Take a snapshot of the orders to match against
                        let orders_to_match: Vec<OrderBookEntry> = sell_orders.clone();
                        
                        for sell_order in orders_to_match {
                            if to_match <= BigDecimal::from(0) {
                                break;
                            }
                            
                            // Calculate match amount
                            let match_amount = to_match.min(sell_order.remain.clone());
                            to_match -= match_amount.clone();
                            
                            // Create and process trade
                            self.create_trade(
                                &order.market_id,
                                &matched_price,
                                &match_amount,
                                &order.id,
                                &order.user_id,
                                &sell_order.order_id,
                                &sell_order.user_id
                            )?;
                            
                            // Update order book
                            let new_remain = sell_order.remain.clone() - match_amount.clone();
                            if new_remain <= BigDecimal::from(0) {
                                order_book.remove_order(&sell_order.order_id, OrderSide::Sell, &matched_price)?;
                            } else {
                                order_book.update_order(&sell_order.order_id, OrderSide::Sell, &matched_price, &new_remain)?;
                            }
                        }
                    }
                }
                
                // Update buyer's order
                let filled_amount = amount.clone() - to_match.clone();
                if filled_amount > BigDecimal::from(0) {
                    self.update_order_after_match(&order.id, &to_match, filled_amount, price)?;
                }
            },
            OrderSide::Sell => {
                // Match against buy orders with price >= sell price
                let mut matched_prices = Vec::new();
                let mut to_match = amount.clone();
                
                // Find all eligible buy prices
                for (&buy_price, _) in order_book.buy_orders.iter() {
                    if &buy_price >= price {
                        matched_prices.push(buy_price.clone());
                    } else {
                        break; // Prices are sorted, so we can break early
                    }
                }
                
                // Match against each price level
                for matched_price in matched_prices {
                    if to_match <= BigDecimal::from(0) {
                        break;
                    }
                    
                    if let Some(buy_orders) = order_book.buy_orders.get_mut(&matched_price) {
                        // Take a snapshot of the orders to match against
                        let orders_to_match: Vec<OrderBookEntry> = buy_orders.clone();
                        
                        for buy_order in orders_to_match {
                            if to_match <= BigDecimal::from(0) {
                                break;
                            }
                            
                            // Calculate match amount
                            let match_amount = to_match.min(buy_order.remain.clone());
                            to_match -= match_amount.clone();
                            
                            // Create and process trade
                            self.create_trade(
                                &order.market_id,
                                &matched_price,
                                &match_amount,
                                &order.id,
                                &order.user_id,
                                &buy_order.order_id,
                                &buy_order.user_id
                            )?;
                            
                            // Update order book
                            let new_remain = buy_order.remain.clone() - match_amount.clone();
                            if new_remain <= BigDecimal::from(0) {
                                order_book.remove_order(&buy_order.order_id, OrderSide::Buy, &matched_price)?;
                            } else {
                                order_book.update_order(&buy_order.order_id, OrderSide::Buy, &matched_price, &new_remain)?;
                            }
                        }
                    }
                }
                
                // Update seller's order
                let filled_amount = amount.clone() - to_match.clone();
                if filled_amount > BigDecimal::from(0) {
                    self.update_order_after_match(&order.id, &to_match, filled_amount, price)?;
                }
            }
        }
        
        Ok(())
    }
    
    // Match a market order against the order book
    fn match_market_order(&self, order_book: &mut OrderBook, order: &Order, 
                         side: OrderSide, amount: &BigDecimal) -> Result<()> {
        match side {
            OrderSide::Buy => {
                // Match against all available sell orders
                let mut matched_prices: Vec<BigDecimal> = order_book.sell_orders.keys().cloned().collect();
                let mut to_match = amount.clone();
                
                // Match against each price level
                for matched_price in matched_prices {
                    if to_match <= BigDecimal::from(0) {
                        break;
                    }
                    
                    if let Some(sell_orders) = order_book.sell_orders.get_mut(&matched_price) {
                        // Take a snapshot of the orders to match against
                        let orders_to_match: Vec<OrderBookEntry> = sell_orders.clone();
                        
                        for sell_order in orders_to_match {
                            if to_match <= BigDecimal::from(0) {
                                break;
                            }
                            
                            // Calculate match amount
                            let match_amount = to_match.min(sell_order.remain.clone());
                            to_match -= match_amount.clone();
                            
                            // Create and process trade
                            self.create_trade(
                                &order.market_id,
                                &matched_price,
                                &match_amount,
                                &order.id,
                                &order.user_id,
                                &sell_order.order_id,
                                &sell_order.user_id
                            )?;
                            
                            // Update order book
                            let new_remain = sell_order.remain.clone() - match_amount.clone();
                            if new_remain <= BigDecimal::from(0) {
                                order_book.remove_order(&sell_order.order_id, OrderSide::Sell, &matched_price)?;
                            } else {
                                order_book.update_order(&sell_order.order_id, OrderSide::Sell, &matched_price, &new_remain)?;
                            }
                        }
                    }
                }
                
                // Update buyer's order status to filled or rejected
                let filled_amount = amount.clone() - to_match.clone();
                if filled_amount > BigDecimal::from(0) {
                    // Mark as filled or partially filled
                    let status = if to_match > BigDecimal::from(0) {
                        OrderStatus::PartiallyFilled.as_str()
                    } else {
                        OrderStatus::Filled.as_str()
                    };
                    
                    self.update_order_after_match(&order.id, &to_match, filled_amount, &BigDecimal::from(0))?;
                } else {
                    // Reject order if no matches
                    self.sender.send(PersistenceMessage::UpdateOrder(
                        order.id.clone(),
                        amount.clone(),
                        BigDecimal::from(0),
                        BigDecimal::from(0),
                        BigDecimal::from(0),
                        BigDecimal::from(0),
                        false,
                        OrderStatus::Rejected.as_str().to_string(),
                    ))?;
                }
            },
            OrderSide::Sell => {
                // Match against all available buy orders
                let mut matched_prices: Vec<BigDecimal> = order_book.buy_orders.keys().cloned().collect();
                let mut to_match = amount.clone();
                
                // Match against each price level
                for matched_price in matched_prices {
                    if to_match <= BigDecimal::from(0) {
                        break;
                    }
                    
                    if let Some(buy_orders) = order_book.buy_orders.get_mut(&matched_price) {
                        // Take a snapshot of the orders to match against
                        let orders_to_match: Vec<OrderBookEntry> = buy_orders.clone();
                        
                        for buy_order in orders_to_match {
                            if to_match <= BigDecimal::from(0) {
                                break;
                            }
                            
                            // Calculate match amount
                            let match_amount = to_match.min(buy_order.remain.clone());
                            to_match -= match_amount.clone();
                            
                            // Create and process trade
                            self.create_trade(
                                &order.market_id,
                                &matched_price,
                                &match_amount,
                                &order.id,
                                &order.user_id,
                                &buy_order.order_id,
                                &buy_order.user_id
                            )?;
                            
                            // Update order book
                            let new_remain = buy_order.remain.clone() - match_amount.clone();
                            if new_remain <= BigDecimal::from(0) {
                                order_book.remove_order(&buy_order.order_id, OrderSide::Buy, &matched_price)?;
                            } else {
                                order_book.update_order(&buy_order.order_id, OrderSide::Buy, &matched_price, &new_remain)?;
                            }
                        }
                    }
                }
                
                // Update seller's order status to filled or rejected
                let filled_amount = amount.clone() - to_match.clone();
                if filled_amount > BigDecimal::from(0) {
                    // Mark as filled or partially filled
                    let status = if to_match > BigDecimal::from(0) {
                        OrderStatus::PartiallyFilled.as_str()
                    } else {
                        OrderStatus::Filled.as_str()
                    };
                    
                    self.update_order_after_match(&order.id, &to_match, filled_amount, &BigDecimal::from(0))?;
                } else {
                    // Reject order if no matches
                    self.sender.send(PersistenceMessage::UpdateOrder(
                        order.id.clone(),
                        amount.clone(),
                        BigDecimal::from(0),
                        BigDecimal::from(0),
                        BigDecimal::from(0),
                        BigDecimal::from(0),
                        false,
                        OrderStatus::Rejected.as_str().to_string(),
                    ))?;
                }
            }
        }
        
        Ok(())
    }
    
    // Create a trade between two orders
    fn create_trade(&self, market_id: &str, price: &BigDecimal, amount: &BigDecimal,
                   taker_order_id: &str, taker_user_id: &str,
                   maker_order_id: &str, maker_user_id: &str) -> Result<()> {
        let quote_amount = price.clone() * amount.clone();
        
        // In a real implementation, fees would be calculated based on user tiers
        let taker_fee = &quote_amount * BigDecimal::from(0.001); // 0.1% fee
        let maker_fee = &quote_amount * BigDecimal::from(0.0005); // 0.05% fee
        
        let trade_id = Uuid::new_v4().to_string();
        let timestamp = chrono::Utc::now().timestamp();
        
        let new_trade = NewTrade {
            id: trade_id,
            timestamp,
            market_id: market_id.to_string(),
            price: price.clone(),
            amount: amount.clone(),
            quote_amount: quote_amount.clone(),
            taker_user_id: taker_user_id.to_string(),
            taker_order_id: taker_order_id.to_string(),
            taker_fee: taker_fee.clone(),
            maker_user_id: maker_user_id.to_string(),
            maker_order_id: maker_order_id.to_string(),
            maker_fee: maker_fee.clone(),
        };
        
        // Send trade to persistence thread pool
        self.sender.send(PersistenceMessage::SaveTrade(new_trade))
            .map_err(|_| anyhow!("Failed to send persistence message"))?;
        
        // Update balances
        // In a real implementation, you'd need to update user balances accordingly
        // This is simplified for the example
        
        Ok(())
    }
    
    // Update an order after matching
    fn update_order_after_match(&self, order_id: &str, remain: &BigDecimal, 
                              filled_amount: BigDecimal, price: &BigDecimal) -> Result<()> {
        let status = if remain <= &BigDecimal::from(0) {
            OrderStatus::Filled.as_str().to_string()
        } else {
            OrderStatus::PartiallyFilled.as_str().to_string()
        };
        
        let filled_quote = filled_amount.clone() * price.clone();
        let filled_fee = filled_quote.clone() * BigDecimal::from(0.001); // Simplified fee calculation
        
        // Send update to persistence thread pool
        self.sender.send(PersistenceMessage::UpdateOrder(
            order_id.to_string(),
            remain.clone(),
            BigDecimal::from(0), // Update frozen amount in a real implementation
            filled_amount,
            filled_quote,
            filled_fee,
            remain > &BigDecimal::from(0),
            status,
        ))?;
        
        Ok(())
    }
    
    // Cancel an order
    pub fn cancel_order(&self, order_id: &str) -> Result<Order> {
        // Get the order
        let order = self.persistence.get_order(order_id)?
            .ok_or_else(|| anyhow!("Order not found: {}", order_id))?;
        
        // Check if order can be canceled
        if order.status != OrderStatus::Open.as_str() && 
           order.status != OrderStatus::PartiallyFilled.as_str() {
            return Err(anyhow!("Cannot cancel order with status: {}", order.status));
        }
        
        // Update order status
        self.sender.send(PersistenceMessage::UpdateOrder(
            order_id.to_string(),
            order.remain.clone(),
            BigDecimal::from(0), // Update frozen amount in a real implementation
            order.filled_base.clone(),
            order.filled_quote.clone(),
            order.filled_fee.clone(),
            order.partially_filled,
            OrderStatus::Canceled.as_str().to_string(),
        ))?;
        
        // Remove from order book
        let market_id = order.market_id.clone();
        let side = order.get_side()?;
        let price = order.price.clone();
        
        let mut order_books = self.order_books.lock().map_err(|_| anyhow!("Order books lock poisoned"))?;
        
        if let Some(order_book) = order_books.get_mut(&market_id) {
            order_book.remove_order(order_id, side, &price)?;
        }
        
        // Return updated order
        let canceled_order = order.clone(); // In a real implementation, get the updated order
        
        Ok(canceled_order)
    }
    
    // Get the order book for a market
    pub fn get_order_book(&self, market_id: &str) -> Result<(Vec<(BigDecimal, BigDecimal)>, Vec<(BigDecimal, BigDecimal)>)> {
        let order_books = self.order_books.lock().map_err(|_| anyhow!("Order books lock poisoned"))?;
        
        let order_book = order_books.get(market_id)
            .ok_or_else(|| anyhow!("Market not found: {}", market_id))?;
        
        // Aggregate orders at each price level
        let mut buy_levels = Vec::new();
        for (price, orders) in &order_book.buy_orders {
            let total_amount: BigDecimal = orders.iter().map(|o| o.remain.clone()).sum();
            buy_levels.push((price.clone(), total_amount));
        }
        
        let mut sell_levels = Vec::new();
        for (price, orders) in &order_book.sell_orders {
            let total_amount: BigDecimal = orders.iter().map(|o| o.remain.clone()).sum();
            sell_levels.push((price.clone(), total_amount));
        }
        
        Ok((buy_levels, sell_levels))
    }
    
    // Shutdown the engine
    pub fn shutdown(self) -> Result<()> {
        info!("Shutting down trading engine");
        
        // Send shutdown message to all workers
        for _ in 0..self.thread_count {
            if let Err(e) = self.sender.send(PersistenceMessage::Shutdown) {
                error!("Failed to send shutdown message: {}", e);
            }
        }
        
        // Wait for all workers to terminate
      //  for handle in self.
    }
}
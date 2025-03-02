use anyhow::Result;
use log::{debug, info};
use std::sync::{Arc, Mutex};

use crate::db::establish_connection_pool;
use crate::repository::Repository;
use crate::{ models::*};

/// ThreadSafePersistence provides a thread-safe way to access the repository
/// for persisting entities to the database.
/// 
#[derive(Debug, Clone)]
pub struct ThreadSafePersistence {
    repository: Arc<Repository>,
    write_lock: Arc<Mutex<()>>,
}

impl ThreadSafePersistence {
    /// Create a new ThreadSafePersistence instance
    pub fn new(database_url: String, pool_size: u32) -> Self {
        let pool = establish_connection_pool(database_url, pool_size);
        Self {
            repository: Arc::new(Repository::new(pool)),
            write_lock: Arc::new(Mutex::new(())),
        }
    }

    /// Clone this instance to share across threads
    pub fn clone_for_thread(&self) -> Self {
        Self {
            repository: Arc::clone(&self.repository),
            write_lock: Arc::clone(&self.write_lock),
        }
    }

    // Read operations (don't need the write lock)

    // Market operations
    pub fn get_market(&self, market_id: &str) -> Result<Option<Market>> {
        debug!("Getting market with ID: {}", market_id);
        self.repository.get_market(market_id)
    }

    pub fn list_markets(&self) -> Result<Vec<Market>> {
        debug!("Listing all markets");
        self.repository.list_markets()
    }

    // Order operations
    pub fn get_order(&self, order_id: &str) -> Result<Option<Order>> {
        debug!("Getting order with ID: {}", order_id);
        self.repository.get_order(order_id)
    }

    pub fn get_open_orders_for_market(&self, market_id: &str) -> Result<Vec<Order>> {
        debug!("Getting open orders for market: {}", market_id);
        self.repository.get_open_orders_for_market(market_id)
    }

    pub fn get_user_orders(&self, user_id: &str, limit: i64) -> Result<Vec<Order>> {
        debug!("Getting orders for user: {} (limit: {})", user_id, limit);
        self.repository.get_user_orders(user_id, limit)
    }

    // Trade operations
    pub fn get_trades_for_market(&self, market_id: &str, limit: i64) -> Result<Vec<Trade>> {
        debug!(
            "Getting trades for market: {} (limit: {})",
            market_id, limit
        );
        self.repository.get_trades_for_market(market_id, limit)
    }

    pub fn get_trades_for_order(&self, order_id: &str) -> Result<Vec<Trade>> {
        debug!("Getting trades for order: {}", order_id);
        self.repository.get_trades_for_order(order_id)
    }

    pub fn get_user_trades(&self, user_id: &str, limit: i64) -> Result<Vec<Trade>> {
        debug!("Getting trades for user: {} (limit: {})", user_id, limit);
        self.repository.get_user_trades(user_id, limit)
    }

    // Balance operations
    pub fn get_balance(&self, user_id: &str, asset: &str) -> Result<Option<Balance>> {
        debug!("Getting balance for user: {}, asset: {}", user_id, asset);
        self.repository.get_balance(user_id, asset)
    }

    // Market stats operations
    pub fn get_market_stats(&self, market_id: &str) -> Result<Option<MarketStat>> {
        debug!("Getting market stats for market: {}", market_id);
        self.repository.get_market_stats(market_id)
    }

    // Write operations (need the write lock)

    // Market operations
    pub fn create_market(&self, market_data: NewMarket) -> Result<Market> {
        let _lock = self
            .write_lock
            .lock()
            .map_err(|e| anyhow::anyhow!("Lock poisoned: {}", e))?;
        info!("Creating new market: {}", market_data.id);
        self.repository.create_market(market_data)
    }

    // Order operations
    pub fn create_order(&self, order_data: NewOrder) -> Result<Order> {
        let _lock = self
            .write_lock
            .lock()
            .map_err(|e| anyhow::anyhow!("Lock poisoned: {}", e))?;
        info!("Creating new order: {}", order_data.id);
        self.repository.create_order(order_data)
    }

    pub fn update_order(
        &self,
        order_id: &str,
        remain: bigdecimal::BigDecimal,
        frozen: bigdecimal::BigDecimal,
        filled_base: bigdecimal::BigDecimal,
        filled_quote: bigdecimal::BigDecimal,
        filled_fee: bigdecimal::BigDecimal,
        partially_filled: bool,
        status: &str,
    ) -> Result<Order> {
        let _lock = self
            .write_lock
            .lock()
            .map_err(|e| anyhow::anyhow!("Lock poisoned: {}", e))?;
        info!("Updating order: {}", order_id);
        self.repository.update_order(
            order_id,
            remain,
            frozen,
            filled_base,
            filled_quote,
            filled_fee,
            partially_filled,
            status,
        )
    }

    // Trade operations
    pub fn create_trade(&self, trade_data: NewTrade) -> Result<Trade> {
        let _lock = self
            .write_lock
            .lock()
            .map_err(|e| anyhow::anyhow!("Lock poisoned: {}", e))?;
        info!("Creating new trade: {}", trade_data.id);
        self.repository.create_trade(trade_data)
    }

    pub fn create_trades(&self, trades_data: Vec<NewTrade>) -> Result<Vec<Trade>> {
        let _lock = self
            .write_lock
            .lock()
            .map_err(|e| anyhow::anyhow!("Lock poisoned: {}", e))?;
        info!("Creating {} new trades", trades_data.len());
        self.repository.create_trades(trades_data)
    }

    // Balance operations
    pub fn update_or_create_balance(
        &self,
        user_id: &str,
        asset: &str,
        available_delta: bigdecimal::BigDecimal,
        frozen_delta: bigdecimal::BigDecimal,
    ) -> Result<Balance> {
        let _lock = self
            .write_lock
            .lock()
            .map_err(|e| anyhow::anyhow!("Lock poisoned: {}", e))?;
        info!("Updating balance for user: {}, asset: {}", user_id, asset);
        self.repository
            .update_or_create_balance(user_id, asset, available_delta, frozen_delta)
    }

    // Market stats operations
    pub fn update_market_stats(
        &self,
        market_id: &str,
        high_24h: bigdecimal::BigDecimal,
        low_24h: bigdecimal::BigDecimal,
        volume_24h: bigdecimal::BigDecimal,
        price_change_24h: bigdecimal::BigDecimal,
        last_price: bigdecimal::BigDecimal,
    ) -> Result<MarketStat> {
        let _lock = self
            .write_lock
            .lock()
            .map_err(|e| anyhow::anyhow!("Lock poisoned: {}", e))?;
        info!("Updating market stats for market: {}", market_id);
        self.repository.update_market_stats(
            market_id,
            high_24h,
            low_24h,
            volume_24h,
            price_change_24h,
            last_price,
        )
    }

    // Transaction support
    pub fn with_transaction<F, T>(&self, operation: F) -> Result<T>
    where
        F: FnOnce() -> Result<T>,
    {
        let _lock = self
            .write_lock
            .lock()
            .map_err(|e| anyhow::anyhow!("Lock poisoned: {}", e))?;
        debug!("Starting database transaction");

        // Execute the operation with the lock held
        let result = operation();

        debug!("Database transaction completed");
        result
    }
}

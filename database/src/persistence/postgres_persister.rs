use anyhow::Result;
use bigdecimal::BigDecimal;
use log::info;
use std::sync::{Arc, Mutex};

use crate::establish_connection_pool;
use crate::models::models::*;
use crate::repository::Repository;

use super::Persistence;
/// PostgresPersister provides a thread-safe way to access the repository
/// for persisting entities to the database.
///
#[derive(Debug, Clone)]
pub struct PostgresPersister {
    repository: Arc<Repository>,
    write_lock: Arc<Mutex<()>>,
}
impl PostgresPersister {
    /// Create a new ThreadSafePersistence instance
    pub fn new(database_url: String, pool_size: u32) -> Self {
        let pool = establish_connection_pool(database_url, pool_size);
        Self {
            repository: Arc::new(Repository::new(pool)),
            write_lock: Arc::new(Mutex::new(())),
        }
    }
}
impl Persistence for PostgresPersister {
    // /// Clone this instance to share across threads
    // fn clone_for_thread(&self) -> Self {
    //     Self {
    //         repository: Arc::clone(&self.repository),
    //         write_lock: Arc::clone(&self.write_lock),
    //     }
    // }

    // Balance operations
    fn get_balance(&self, user_id: &str, asset: &str) -> Result<Option<Wallet>> {
        self.repository.get_wallet(user_id, asset)
    }

    // Market operations
    fn get_market(&self, market_id: &str) -> Result<Option<Market>> {
        self.repository.get_market(market_id)
    }

    fn list_markets(&self) -> Result<Vec<Market>> {
        self.repository.list_markets()
    }

    // Order operations
    fn get_order(&self, order_id: &str) -> Result<Option<Order>> {
        self.repository.get_order(order_id)
    }

    fn get_open_orders_for_market(&self, market_id: &str) -> Result<Vec<Order>> {
        self.repository.get_open_orders_for_market(market_id)
    }

    fn get_user_orders(&self, user_id: &str, limit: i64) -> Result<Vec<Order>> {
        self.repository.get_user_orders(user_id, limit)
    }

    //wallet operations
    fn deposit_balance(&self, user_id: &str, asset: &str, amount: BigDecimal) -> Result<Wallet> {
        self.repository.deposit_balance(user_id, asset, amount)
    }
    fn withdraw_balance(&self, user_id: &str, asset: &str, amount: BigDecimal) -> Result<Wallet> {
        self.repository.withdraw_balance(user_id, asset, amount)
    }
    fn lock_balance(&self, user_id: &str, asset: &str, amount: BigDecimal) -> Result<Wallet> {
        self.repository.lock_balance(user_id, asset, amount)
    }
    fn unlock_balance(&self, user_id: &str, asset: &str, amount: BigDecimal) -> Result<Wallet> {
        self.repository.unlock_balance(user_id, asset, amount)
    }
    // Trade operations
    fn get_trades_for_market(&self, market_id: &str, limit: i64) -> Result<Vec<Trade>> {
        self.repository.get_trades_for_market(market_id, limit)
    }

    fn get_trades_for_order(&self, order_id: &str) -> Result<Vec<Trade>> {
        self.repository.get_trades_for_order(order_id)
    }

    fn get_user_trades(&self, user_id: &str, limit: i64) -> Result<Vec<Trade>> {
        self.repository.get_user_trades(user_id, limit)
    }

    // Market stats operations
    fn get_market_stats(&self, market_id: &str) -> Result<Option<MarketStat>> {
        self.repository.get_market_stats(market_id)
    }

    // Write operations (need the write lock)

    // Market operations
    fn create_market(&self, market_data: NewMarket) -> Result<Market> {
        let _lock = self
            .write_lock
            .lock()
            .map_err(|e| anyhow::anyhow!("Lock poisoned: {}", e))?;
        info!("Creating new market: {}", market_data.id);
        self.repository.create_market(market_data)
    }

    // Order operations
    fn create_order(&self, order_data: NewOrder) -> Result<Order> {
        let _lock = self
            .write_lock
            .lock()
            .map_err(|e| anyhow::anyhow!("Lock poisoned: {}", e))?;
        info!("Creating new order: {}", order_data.id);
        self.repository.create_order(order_data)
    }

    // Market stats operations
    fn update_market_stats(
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
    fn execute_limit_trade(
        &self,
        is_buyer_taker: bool,
        market_id: String,
        base_asset: String,
        quote_asset: String,
        buyer_user_id: String,
        seller_user_id: String,
        buyer_order_id: String,
        seller_order_id: String,
        price: BigDecimal,
        base_amount: BigDecimal,
        trade_quote_amount: BigDecimal,
        buyer_fee: BigDecimal,
        seller_fee: BigDecimal,
    ) -> Result<NewTrade> {
        let _lock = self
            .write_lock
            .lock()
            .map_err(|e| anyhow::anyhow!("Lock poisoned: {}", e))?;
        self.repository.execute_limit_trade(
            is_buyer_taker,
            market_id,
            base_asset,
            quote_asset,
            buyer_user_id,
            seller_user_id,
            buyer_order_id,
            seller_order_id,
            price,
            base_amount,
            trade_quote_amount,
            buyer_fee,
            seller_fee,
        )
    }

    fn cancel_order(&self, order_id: &str) -> Result<Order> {
        let _lock = self
            .write_lock
            .lock()
            .map_err(|e| anyhow::anyhow!("Lock poisoned: {}", e))?;
        self.repository.cancel_order(order_id)
    }

    fn cancel_all_orders(&self, market_id: &str) -> Result<Vec<Order>> {
        let _lock = self
            .write_lock
            .lock()
            .map_err(|e| anyhow::anyhow!("Lock poisoned: {}", e))?;
        self.repository.cancel_all_orders(market_id)
    }

    fn cancel_all_global_orders(&self) -> Result<Vec<Order>> {
        let _lock = self
            .write_lock
            .lock()
            .map_err(|e| anyhow::anyhow!("Lock poisoned: {}", e))?;
        self.repository.cancel_all_global_orders()
    }

    fn get_active_orders(&self, market_id: &str) -> Result<Vec<Order>> {
        let _lock = self
            .write_lock
            .lock()
            .map_err(|e| anyhow::anyhow!("Lock poisoned: {}", e))?;
        self.repository.get_active_orders(market_id)
    }

    fn get_all_active_orders(&self) -> Result<Vec<Order>> {
        let _lock = self
            .write_lock
            .lock()
            .map_err(|e| anyhow::anyhow!("Lock poisoned: {}", e))?;
        self.repository.get_all_active_orders()
    }
    fn get_user_active_orders_count(&self, market_id: &str, user_id: &str) -> Result<Vec<Order>> {
        let _lock = self
            .write_lock
            .lock()
            .map_err(|e| anyhow::anyhow!("Lock poisoned: {}", e))?;
        self.repository
            .get_user_active_orders_count(market_id, user_id)
    }
    // Transaction support
    fn with_transaction<F, T>(&self, operation: F) -> Result<T>
    where
        F: FnOnce() -> Result<T>,
    {
        let _lock = self
            .write_lock
            .lock()
            .map_err(|e| anyhow::anyhow!("Lock poisoned: {}", e))?;

        let result = operation();

        result
    }
}

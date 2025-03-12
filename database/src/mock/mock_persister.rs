use anyhow::Result;
use bigdecimal::BigDecimal;
use chrono::Utc;
use std::collections::HashMap;
use std::sync::{Arc, Mutex};

use crate::models::models::*;
use crate::persistence::Persistence;
#[derive(Debug, Clone)]
pub struct MockPersister {
    markets: Arc<Mutex<HashMap<String, Market>>>,
    orders: Arc<Mutex<HashMap<String, Order>>>,
    balances: Arc<Mutex<HashMap<(String, String), Wallet>>>,
    trades: Arc<Mutex<HashMap<String, Trade>>>,
    market_stats: Arc<Mutex<HashMap<String, MarketStat>>>,
}

impl MockPersister {
    pub fn new() -> Self {
        Self {
            markets: Arc::new(Mutex::new(HashMap::new())),
            orders: Arc::new(Mutex::new(HashMap::new())),
            balances: Arc::new(Mutex::new(HashMap::new())),
            trades: Arc::new(Mutex::new(HashMap::new())),
            market_stats: Arc::new(Mutex::new(HashMap::new())),
        }
    }

    pub fn insert_market(&self, market: Market) {
        self.markets
            .lock()
            .unwrap()
            .insert(market.id.clone(), market);
    }

    pub fn insert_order(&self, order: Order) {
        self.orders.lock().unwrap().insert(order.id.clone(), order);
    }

    pub fn insert_balance(&self, balance: Wallet) {
        self.balances
            .lock()
            .unwrap()
            .insert((balance.user_id.clone(), balance.asset.clone()), balance);
    }

    // Add a new field to store trades and market stats
    pub fn with_trades_and_stats() -> Self {
        Self {
            markets: Arc::new(Mutex::new(HashMap::new())),
            orders: Arc::new(Mutex::new(HashMap::new())),
            balances: Arc::new(Mutex::new(HashMap::new())),
            trades: Arc::new(Mutex::new(HashMap::new())),
            market_stats: Arc::new(Mutex::new(HashMap::new())),
        }
    }

    // Helper method to insert a trade
    pub fn insert_trade(&self, trade: Trade) {
        self.trades.lock().unwrap().insert(trade.id.clone(), trade);
    }

    // Helper method to insert market stats
    pub fn insert_market_stat(&self, market_stat: MarketStat) {
        self.market_stats
            .lock()
            .unwrap()
            .insert(market_stat.market_id.clone(), market_stat);
    }
}

impl Persistence for MockPersister {
    fn get_market(&self, market_id: &str) -> Result<Option<Market>> {
        Ok(self.markets.lock().unwrap().get(market_id).cloned())
    }

    fn list_markets(&self) -> Result<Vec<Market>> {
        Ok(self.markets.lock().unwrap().values().cloned().collect())
    }

    fn get_order(&self, order_id: &str) -> Result<Option<Order>> {
        Ok(self.orders.lock().unwrap().get(order_id).cloned())
    }
    fn get_open_orders_for_market(&self, market_id: &str) -> Result<Vec<Order>> {
        let orders = self
            .orders
            .lock()
            .unwrap()
            .values()
            .filter(|o| o.market_id == market_id)
            .cloned()
            .collect();
        Ok(orders)
    }

    fn get_user_orders(&self, user_id: &str, _limit: i64) -> Result<Vec<Order>> {
        let orders = self
            .orders
            .lock()
            .unwrap()
            .values()
            .filter(|o| o.user_id == user_id)
            .cloned()
            .collect();
        Ok(orders)
    }

    fn get_balance(&self, user_id: &str, asset: &str) -> Result<Option<Wallet>> {
        Ok(self
            .balances
            .lock()
            .unwrap()
            .get(&(user_id.to_string(), asset.to_string()))
            .cloned())
    }

    // Trade operations
    fn get_trades_for_market(&self, market_id: &str, limit: i64) -> Result<Vec<Trade>> {
        let trades = self.trades.lock().unwrap();
        let mut result: Vec<Trade> = trades
            .values()
            .filter(|t| t.market_id == market_id)
            .cloned()
            .collect();

        // // Sort by created_at in descending order (newest first)
        // result.sort_by(|a, b| b.created_at.cmp(&a.created_at));

        // Apply limit if needed
        if limit > 0 && result.len() > limit as usize {
            result.truncate(limit as usize);
        }

        Ok(result)
    }

    fn get_trades_for_order(&self, order_id: &str) -> Result<Vec<Trade>> {
        let trades = self.trades.lock().unwrap();
        let result: Vec<Trade> = trades
            .values()
            .filter(|t| t.buyer_order_id == order_id || t.seller_order_id == order_id)
            .cloned()
            .collect();

        Ok(result)
    }

    fn get_user_trades(&self, user_id: &str, limit: i64) -> Result<Vec<Trade>> {
        let trades = self.trades.lock().unwrap();
        let mut result: Vec<Trade> = trades
            .values()
            .filter(|t| t.buyer_user_id == user_id || t.seller_user_id == user_id)
            .cloned()
            .collect();

        // // Sort by created_at in descending order (newest first)
        // result.sort_by(|a, b| b.created_at.cmp(&a.created_at));

        // Apply limit if needed
        if limit > 0 && result.len() > limit as usize {
            result.truncate(limit as usize);
        }

        Ok(result)
    }

    // Market stats operations
    fn get_market_stats(&self, market_id: &str) -> Result<Option<MarketStat>> {
        Ok(self.market_stats.lock().unwrap().get(market_id).cloned())
    }

    // Write operations
    // Market operations
    fn create_market(&self, market_data: NewMarket) -> Result<Market> {
        let market = Market {
            id: market_data.id,
            base_asset: market_data.base_asset,
            quote_asset: market_data.quote_asset,
            default_maker_fee: market_data.default_maker_fee,
            default_taker_fee: market_data.default_taker_fee,
            create_time: market_data.create_time,
            update_time: market_data.update_time,
            amount_precision: market_data.amount_precision,
            min_base_amount: market_data.min_base_amount,
            min_quote_amount: market_data.min_quote_amount,
            price_precision: market_data.price_precision,
            status: market_data.status,
        };

        self.insert_market(market.clone());
        Ok(market)
    }

    //wallet operations
    fn deposit_balance(&self, user_id: &str, asset: &str, amount: BigDecimal) -> Result<Wallet> {
        unimplemented!()
    }
    fn withdraw_balance(&self, user_id: &str, asset: &str, amount: BigDecimal) -> Result<Wallet> {
        unimplemented!()
    }
    fn lock_balance(&self, user_id: &str, asset: &str, amount: BigDecimal) -> Result<Wallet> {
        unimplemented!()
    }
    fn unlock_balance(&self, user_id: &str, asset: &str, amount: BigDecimal) -> Result<Wallet> {
        unimplemented!()
    }
    // Order operations
    fn create_order(&self, order_data: NewOrder) -> Result<Order> {
        let order = Order {
            id: order_data.id,
            user_id: order_data.user_id,
            market_id: order_data.market_id,
            side: order_data.side,
            price: order_data.price,
            base_amount: order_data.base_amount,
            quote_amount: order_data.quote_amount,
            remained_base: order_data.remained_base,
            remained_quote: order_data.remained_quote,
            filled_base: order_data.filled_base,
            filled_quote: order_data.filled_quote,
            filled_fee: order_data.filled_fee,
            status: order_data.status,
            order_type: order_data.order_type,
            maker_fee: order_data.maker_fee,
            taker_fee: order_data.taker_fee,
            client_order_id: order_data.client_order_id,
            expires_at: order_data.expires_at,
            post_only: order_data.post_only,
            time_in_force: order_data.time_in_force,
            create_time: Utc::now().timestamp_millis(),
            update_time: Utc::now().timestamp_millis(),
        };

        self.insert_order(order.clone());
        Ok(order)
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
        let market_stat = MarketStat {
            market_id: market_id.to_string(),
            high_24h,
            low_24h,
            volume_24h,
            price_change_24h,
            last_price,
            last_update_time: Utc::now().timestamp_millis(),
        };

        self.insert_market_stat(market_stat.clone());
        Ok(market_stat)
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
        unimplemented!()
    }

    fn cancel_order(&self, order_id: &str) -> Result<Order> {
        unimplemented!()
    }

    fn cancel_all_orders(&self, market_id: &str) -> Result<Vec<Order>> {
        unimplemented!()
    }

    fn cancel_all_global_orders(&self) -> Result<Vec<Order>> {
        unimplemented!()
    }

    fn get_active_orders(&self, market_id: &str) -> Result<Vec<Order>> {
        unimplemented!()
    }

    fn get_all_active_orders(&self) -> Result<Vec<Order>> {
        unimplemented!()
    }
    fn get_user_active_orders_count(&self, market_id: &str, user_id: &str) -> Result<Vec<Order>> {
        unimplemented!()
    }
    // Transaction support
    fn with_transaction<F, T>(&self, operation: F) -> Result<T>
    where
        F: FnOnce() -> Result<T>,
    {
        // For mock implementation, just execute the operation directly
        // without actual transaction support
        operation()
    }
}

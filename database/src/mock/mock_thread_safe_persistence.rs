use anyhow::Result;
use chrono::Utc;
use std::collections::HashMap;
use std::sync::{Arc, Mutex};

use crate::models::models::*;
use crate::persistence::persistence::Persistence;
#[derive(Debug, Clone)]
pub struct MockThreadSafePersistence {
    markets: Arc<Mutex<HashMap<String, Market>>>,
    orders: Arc<Mutex<HashMap<String, Order>>>,
    balances: Arc<Mutex<HashMap<(String, String), Balance>>>,
    trades: Arc<Mutex<HashMap<String, Trade>>>,
    market_stats: Arc<Mutex<HashMap<String, MarketStat>>>,
}

impl MockThreadSafePersistence {
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

    pub fn insert_balance(&self, balance: Balance) {
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

impl Persistence for MockThreadSafePersistence {
    fn get_market(&self, market_id: &str) -> Result<Option<Market>> {
        Ok(self.markets.lock().unwrap().get(market_id).cloned())
    }

    fn list_markets(&self) -> Result<Vec<Market>> {
        Ok(self.markets.lock().unwrap().values().cloned().collect())
    }

    fn get_order(&self, order_id: &str) -> Result<Option<Order>> {
        Ok(self.orders.lock().unwrap().get(order_id).cloned())
    }
    fn update_order(
        &self,
        order_id: &str,
        remain: bigdecimal::BigDecimal,
        filled_base: bigdecimal::BigDecimal,
        filled_quote: bigdecimal::BigDecimal,
        filled_fee: bigdecimal::BigDecimal,
        status: &str,
    ) -> Result<Order> {
        let mut orders = self.orders.lock().unwrap();

        if let Some(mut order) = orders.get(order_id).cloned() {
            order.remain = remain;
            order.filled_base = filled_base;
            order.filled_quote = filled_quote;
            order.filled_fee = filled_fee;

            order.status = status.to_string();

            orders.insert(order_id.to_string(), order.clone());
            Ok(order)
        } else {
            Err(anyhow::anyhow!("Order not found"))
        }
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

    fn get_balance(&self, user_id: &str, asset: &str) -> Result<Option<Balance>> {
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
            .filter(|t| t.maker_order_id == order_id || t.taker_order_id == order_id)
            .cloned()
            .collect();

        Ok(result)
    }

    fn get_user_trades(&self, user_id: &str, limit: i64) -> Result<Vec<Trade>> {
        let trades = self.trades.lock().unwrap();
        let mut result: Vec<Trade> = trades
            .values()
            .filter(|t| t.maker_user_id == user_id || t.taker_user_id == user_id)
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
        };

        self.insert_market(market.clone());
        Ok(market)
    }

    // Order operations
    fn create_order(&self, order_data: NewOrder) -> Result<Order> {
        let order = Order {
            id: order_data.id,
            user_id: order_data.user_id,
            market_id: order_data.market_id,
            side: order_data.side,
            price: order_data.price,
            amount: order_data.amount,
            remain: order_data.remain,

            filled_base: order_data.filled_base,
            filled_quote: order_data.filled_quote,
            filled_fee: order_data.filled_fee,
            status: order_data.status,
            order_type: order_data.order_type,
            maker_fee: order_data.maker_fee,
            taker_fee: order_data.taker_fee,
            create_time: Utc::now().timestamp_millis(),
            update_time: Utc::now().timestamp_millis(),
        };

        self.insert_order(order.clone());
        Ok(order)
    }

    // Trade operations
    fn create_trade(&self, trade_data: NewTrade) -> Result<Trade> {
        let trade = Trade {
            id: trade_data.id,
            market_id: trade_data.market_id,
            maker_order_id: trade_data.maker_order_id,
            taker_order_id: trade_data.taker_order_id,
            maker_user_id: trade_data.maker_user_id,
            taker_user_id: trade_data.taker_user_id,
            price: trade_data.price,
            amount: trade_data.amount,
            quote_amount: trade_data.quote_amount,
            maker_fee: trade_data.maker_fee,
            taker_fee: trade_data.taker_fee,
            timestamp: Utc::now().timestamp_millis(),
        };

        self.insert_trade(trade.clone());
        Ok(trade)
    }

    fn create_trades(&self, trades_data: Vec<NewTrade>) -> Result<Vec<Trade>> {
        let mut result = Vec::new();

        for trade_data in trades_data {
            let trade = self.create_trade(trade_data)?;
            result.push(trade);
        }

        Ok(result)
    }

    // Balance operations
    fn update_or_create_balance(
        &self,
        user_id: &str,
        asset: &str,
        available_delta: bigdecimal::BigDecimal,
        frozen_delta: bigdecimal::BigDecimal,
    ) -> Result<Balance> {
        let mut balances = self.balances.lock().unwrap();
        let key = (user_id.to_string(), asset.to_string());

        if let Some(mut balance) = balances.get(&key).cloned() {
            balance.available = balance.available.clone() + available_delta.clone();
            balance.frozen = balance.frozen.clone() + frozen_delta.clone();
            balance.update_time = chrono::Utc::now().timestamp_millis();

            balances.insert(key, balance.clone());
            Ok(balance)
        } else {
            let balance = Balance {
                user_id: user_id.to_string(),
                asset: asset.to_string(),
                available: available_delta,
                frozen: frozen_delta,
                update_time: Utc::now().timestamp_millis(),
            };

            balances.insert(key, balance.clone());
            Ok(balance)
        }
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

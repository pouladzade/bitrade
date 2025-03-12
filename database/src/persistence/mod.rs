pub mod postgres_persister;

use crate::models::models::*;
use anyhow::Result;
use bigdecimal::BigDecimal;
use std::fmt::Debug;

pub trait Persistence: Send + Sync + Clone + Debug {
    fn get_market(&self, market_id: &str) -> Result<Option<Market>>;
    fn list_markets(&self) -> Result<Vec<Market>>;

    fn get_order(&self, order_id: &str) -> Result<Option<Order>>;
    fn get_open_orders_for_market(&self, market_id: &str) -> Result<Vec<Order>>;
    fn get_user_orders(&self, user_id: &str, limit: i64) -> Result<Vec<Order>>;

    fn get_balance(&self, user_id: &str, asset: &str) -> Result<Option<Wallet>>;
    // Trade operations
    fn get_trades_for_market(&self, market_id: &str, limit: i64) -> Result<Vec<Trade>>;
    fn get_trades_for_order(&self, order_id: &str) -> Result<Vec<Trade>>;
    fn get_user_trades(&self, user_id: &str, limit: i64) -> Result<Vec<Trade>>;

    // Market stats operations
    fn get_market_stats(&self, market_id: &str) -> Result<Option<MarketStat>>;

    // Market operations
    fn create_market(&self, market_data: NewMarket) -> Result<Market>;

    // Order operations
    fn create_order(&self, order_data: NewOrder) -> Result<Order>;

    //wallet operations
    fn deposit_balance(&self, user_id: &str, asset: &str, amount: BigDecimal) -> Result<Wallet>;
    fn withdraw_balance(&self, user_id: &str, asset: &str, amount: BigDecimal) -> Result<Wallet>;
    fn lock_balance(&self, user_id: &str, asset: &str, amount: BigDecimal) -> Result<Wallet>;
    fn unlock_balance(&self, user_id: &str, asset: &str, amount: BigDecimal) -> Result<Wallet>;

    // Market stats operations
    fn update_market_stats(
        &self,
        market_id: &str,
        high_24h: BigDecimal,
        low_24h: BigDecimal,
        volume_24h: BigDecimal,
        price_change_24h: BigDecimal,
        last_price: BigDecimal,
    ) -> Result<MarketStat>;
    fn cancel_order(&self, order_id: &str) -> Result<Order>;
    fn cancel_all_orders(&self, market_id: &str) -> Result<Vec<Order>>;
    fn cancel_all_global_orders(&self) -> Result<Vec<Order>>;
    fn get_active_orders(&self, market_id: &str) -> Result<Vec<Order>>;
    fn get_all_active_orders(&self) -> Result<Vec<Order>>;
    fn get_user_active_orders_count(&self, market_id: &str, user_id: &str) -> Result<Vec<Order>>;
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
    ) -> Result<NewTrade>;
    // Transaction support
    fn with_transaction<F, T>(&self, operation: F) -> Result<T>
    where
        F: FnOnce() -> Result<T>;
}

use crate::models::*;
use anyhow::Result;
use bigdecimal::BigDecimal;

pub trait DatabaseReader {
    // Market Reads
    fn get_market(&self, market_id: &str) -> Result<Option<Market>>;
    fn list_markets(&self) -> Result<Vec<Market>>;

    // Order Reads
    fn get_order(&self, order_id: &str) -> Result<Option<Order>>;
    fn get_open_orders_for_market(&self, market_id: &str) -> Result<Vec<Order>>;
    fn get_user_orders(&self, user_id: &str, limit: i64) -> Result<Vec<Order>>;

    // Trade Reads
    fn get_trades_for_market(&self, market_id: &str, limit: i64) -> Result<Vec<Trade>>;
    fn get_trades_for_order(&self, order_id: &str) -> Result<Vec<Trade>>;
    fn get_user_trades(&self, user_id: &str, limit: i64) -> Result<Vec<Trade>>;

    // Balance Reads
    fn get_balance(&self, user_id: &str, asset: &str) -> Result<Option<Balance>>;

    // Market Stats Reads
    fn get_market_stats(&self, market_id: &str) -> Result<Option<MarketStat>>;
}

pub trait DatabaseWriter {
    // Market Writes
    fn create_market(&self, market_data: NewMarket) -> Result<Market>;

    // Order Writes
    fn create_order(&self, order_data: NewOrder) -> Result<Order>;
    fn update_order(
        &self,
        order_id: &str,
        remain: BigDecimal,
        frozen: BigDecimal,
        filled_base: BigDecimal,
        filled_quote: BigDecimal,
        filled_fee: BigDecimal,
        partially_filled: bool,
        status: &str,
    ) -> Result<Order>;

    // Trade Writes
    fn create_trade(&self, trade_data: NewTrade) -> Result<Trade>;

    // Balance Writes
    fn update_or_create_balance(
        &self,
        user_id: &str,
        asset: &str,
        available_delta: BigDecimal,
        frozen_delta: BigDecimal,
    ) -> Result<Balance>;

    // Market Stats Writes
    fn update_market_stats(
        &self,
        market_id: &str,
        high_24h: BigDecimal,
        low_24h: BigDecimal,
        volume_24h: BigDecimal,
        price_change_24h: BigDecimal,
        last_price: BigDecimal,
    ) -> Result<MarketStat>;
}

pub trait DatabaseProvider: DatabaseWriter + DatabaseReader {}

impl<T: DatabaseWriter + DatabaseReader> DatabaseProvider for T {}

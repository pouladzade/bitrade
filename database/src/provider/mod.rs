use crate::DbConnection;
use crate::DbPool;
use crate::filters::OrderFilter;
use crate::filters::WalletFilter;
use crate::{filters::TradeFilter, models::models::*};
use anyhow::Result;
use bigdecimal::BigDecimal;
use common::db::pagination::*;

pub trait OrderDatabaseReader {
    fn get_order(&self, order_id: &str) -> Result<Option<Order>>;
    fn get_active_orders(&self, market_id: &str) -> Result<Vec<Order>>;
    fn list_orders(
        &self,
        filter: OrderFilter,
        pagination: Option<Pagination>,
    ) -> Result<Paginated<Order>>;
}

pub trait OrderDatabaseWriter {
    fn create_order(&self, order_data: NewOrder) -> Result<Order>;
    fn cancel_order(&self, order_id: &str) -> Result<Order>;
    fn cancel_all_orders(&self, market_id: &str) -> Result<Vec<Order>>;
    fn cancel_all_global_orders(&self) -> Result<Vec<Order>>;
    fn update_order_status(&self, order_id: &str, status: OrderStatus) -> Result<Order>;
}

pub trait WalletDatabaseReader {
    fn get_wallet(&self, user_id: &str, asset: &str) -> Result<Option<Wallet>>;
    fn list_wallets(
        &self,
        filter: WalletFilter,
        pagination: Option<Pagination>,
    ) -> Result<Paginated<Wallet>>;
}

pub trait WalletDatabaseWriter {
    fn deposit_balance(&self, user_id: &str, asset: &str, amount: BigDecimal) -> Result<Wallet>;
    fn withdraw_balance(&self, user_id: &str, asset: &str, amount: BigDecimal) -> Result<Wallet>;
    fn lock_balance(&self, user_id: &str, asset: &str, amount: BigDecimal) -> Result<Wallet>;
    fn unlock_balance(&self, user_id: &str, asset: &str, amount: BigDecimal) -> Result<Wallet>;
}

pub trait TradeDatabaseReader {
    fn list_trades(
        &self,
        filter: TradeFilter,
        pagination: Option<Pagination>,
    ) -> Result<Paginated<Trade>>;
}

pub trait TradeDatabaseWriter {
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
        quote_amount: BigDecimal,
        buyer_fee_rate: BigDecimal,
        seller_fee_rate: BigDecimal,
    ) -> Result<NewTrade>;
}

pub trait MarketDatabaseReader {
    fn get_market(&self, market_id: &str) -> Result<Option<Market>>;
    fn list_markets(&self) -> Result<Vec<Market>>;
}

pub trait MarketDatabaseWriter {
    fn create_market(&self, market_data: NewMarket) -> Result<Market>;
}

pub trait MarketStatDatabaseReader {
    fn get_market_stats(&self, market_id: &str) -> Result<Option<MarketStat>>;
}

pub trait MarketStatDatabaseWriter {
    fn upsert_market_stats(
        &self,
        market_id: &str,
        high_24h: BigDecimal,
        low_24h: BigDecimal,
        volume_24h: BigDecimal,
        price_change_24h: BigDecimal,
        last_price: BigDecimal,
    ) -> Result<MarketStat>;
}

pub trait FeeTreasuryDatabaseReader {
    fn get_fee_treasury(&self, market_id: &str) -> Result<Option<FeeTreasury>>;
    fn list_fee_treasuries(&self) -> Result<Vec<FeeTreasury>>;
}

pub trait FeeTreasuryDatabaseWriter {
    fn create_fee_treasury(&self, fee_treasury_data: NewFeeTreasury) -> Result<FeeTreasury>;
    fn transfer_to_fee_treasury(&self, fee_amount: BigDecimal) -> Result<FeeTreasury>;
}

pub trait ReadDatabaseProvider:
    Send
    + Sync
    + OrderDatabaseReader
    + WalletDatabaseReader
    + TradeDatabaseReader
    + MarketDatabaseReader
    + MarketStatDatabaseReader
    + FeeTreasuryDatabaseReader
{
}

pub trait WriteDatabaseProvider:
    Send
    + Sync
    + OrderDatabaseWriter
    + WalletDatabaseWriter
    + TradeDatabaseWriter
    + MarketDatabaseWriter
    + MarketStatDatabaseWriter
    + FeeTreasuryDatabaseWriter
{
}

impl<
    T: Send
        + Sync
        + OrderDatabaseReader
        + WalletDatabaseReader
        + TradeDatabaseReader
        + MarketDatabaseReader
        + MarketStatDatabaseReader
        + FeeTreasuryDatabaseReader,
> ReadDatabaseProvider for T
{
}

impl<
    T: Send
        + Sync
        + OrderDatabaseWriter
        + WalletDatabaseWriter
        + TradeDatabaseWriter
        + MarketDatabaseWriter
        + MarketStatDatabaseWriter
        + FeeTreasuryDatabaseWriter,
> WriteDatabaseProvider for T
{
}

pub trait DatabaseProvider: ReadDatabaseProvider + WriteDatabaseProvider + Send + Sync {}

impl<T: ReadDatabaseProvider + WriteDatabaseProvider + Send + Sync> DatabaseProvider for T {}

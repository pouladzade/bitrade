pub mod orders;
pub mod queries;
pub mod trades;
pub mod wallets;

use diesel::r2d2::{ConnectionManager, Pool};
use diesel::PgConnection;

pub type DbPool = Pool<ConnectionManager<PgConnection>>;

// Repository traits
pub trait OrderRepository {
    fn create_order(&self, order: &NewOrder) -> Result<Order>;
    fn get_order(&self, id: &str) -> Result<Option<Order>>;
    fn update_order_status(&self, id: &str, status: OrderStatus) -> Result<Order>;
    // ... other order-specific methods
}

pub trait TradeRepository {
    fn create_trade(&self, trade: &NewTrade) -> Result<Trade>;
    fn get_trade(&self, id: &str) -> Result<Option<Trade>>;
    // ... other trade-specific methods
}

pub trait WalletRepository {
    fn get_balance(&self, user_id: &str, asset: &str) -> Result<Option<Balance>>;
    fn update_balance(&self, update: BalanceUpdate) -> Result<Balance>;
    // ... other wallet-specific methods
}

pub trait QueryRepository {
    fn get_user_orders(&self, user_id: &str, limit: i64) -> Result<Vec<Order>>;
    fn get_market_trades(&self, market_id: &str, limit: i64) -> Result<Vec<Trade>>;
    fn get_user_balances(&self, user_id: &str) -> Result<Vec<Balance>>;
}

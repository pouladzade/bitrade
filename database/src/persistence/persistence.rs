use std::fmt::Debug;

use crate::models::models::*;
use anyhow::Result;

pub trait Persistence: Send + Sync + Clone + Debug {
    fn get_market(&self, market_id: &str) -> Result<Option<Market>>;
    fn list_markets(&self) -> Result<Vec<Market>>;

    fn get_order(&self, order_id: &str) -> Result<Option<Order>>;
    fn get_open_orders_for_market(&self, market_id: &str) -> Result<Vec<Order>>;
    fn get_user_orders(&self, user_id: &str, limit: i64) -> Result<Vec<Order>>;

    fn get_balance(&self, user_id: &str, asset: &str) -> Result<Option<Balance>>;
}

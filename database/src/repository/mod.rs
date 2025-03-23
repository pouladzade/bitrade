mod fee_treasury;
mod market_stats;
mod markets;
mod orders;
mod trades;
mod wallets;

use crate::DbConnection;
use crate::DbPool;
use anyhow::Result;

#[derive(Debug, Clone)]
pub struct Repository {
    pool: DbPool,
}
impl Repository {
    pub fn new(pool: DbPool) -> Self {
        Self { pool }
    }
    pub fn get_conn(&self) -> Result<DbConnection> {
        Ok(self.pool.get()?)
    }
}

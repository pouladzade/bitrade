use super::*;
use crate::models::*;
use crate::schema::*;
use diesel::prelude::*;

pub struct TradeRepositoryImpl {
    pool: DbPool,
}

impl TradeRepositoryImpl {
    pub fn new(pool: DbPool) -> Self {
        Self { pool }
    }
}

impl TradeRepository for TradeRepositoryImpl {
    fn create_trade(&self, trade: &NewTrade) -> Result<Trade> {
        let mut conn = self.pool.get()?;

        diesel::insert_into(trades::table)
            .values(trade)
            .get_result(&mut conn)
            .map_err(Into::into)
    }

}

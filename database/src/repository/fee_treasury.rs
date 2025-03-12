use super::Repository;
use crate::models::models::*;

use crate::models::schema::*;

use anyhow::Result;
use bigdecimal::BigDecimal;

use diesel::prelude::*;

impl Repository {
    // Fee treasury operations
    pub fn create_fee_treasury(
        &self,
        fee_treasury_data: crate::models::models::NewFeeTreasury,
    ) -> Result<crate::models::models::FeeTreasury> {
        let conn = &mut self.get_conn()?;

        let result = diesel::insert_into(fee_treasury::table)
            .values(&fee_treasury_data)
            .get_result(conn)?;

        Ok(result)
    }

    pub fn get_fee_treasury(&self, market_id: &str) -> Result<Option<FeeTreasury>> {
        let conn = &mut self.get_conn()?;

        let result = fee_treasury::table
            .filter(fee_treasury::market_id.eq(market_id))
            .first(conn)
            .optional()?;

        Ok(result)
    }

    pub fn transfer_to_fee_treasury(&self, fee_amount: BigDecimal) -> Result<FeeTreasury> {
        let conn = &mut self.get_conn()?;

        let result = diesel::update(fee_treasury::table)
            .set((
                fee_treasury::collected_amount.eq(fee_amount),
                fee_treasury::last_update_time.eq(chrono::Utc::now().timestamp_millis()),
            ))
            .get_result(conn)?;

        Ok(result)
    }
}

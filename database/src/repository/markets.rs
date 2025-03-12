use super::Repository;
use crate::models::models::*;
use crate::models::schema::*;
use anyhow::Result;
use diesel::prelude::*;
use uuid::Uuid;
impl Repository {
    // Market operations
    pub fn create_market(&self, market_data: NewMarket) -> Result<Market> {
        let conn = &mut self.get_conn()?;
        let result = diesel::insert_into(markets::table)
            .values(market_data)
            .get_result(conn)?;

        Ok(result)
    }
    pub fn get_market(&self, market_id: &str) -> Result<Option<Market>> {
        let conn = &mut self.get_conn()?;

        let result = markets::table.find(market_id).first(conn).optional()?;

        Ok(result)
    }

    pub fn list_markets(&self) -> Result<Vec<Market>> {
        let conn = &mut self.get_conn()?;

        let result = markets::table.load(conn)?;

        Ok(result)
    }
}

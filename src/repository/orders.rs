use super::*;
use crate::models::*;
use crate::schema::*;
use diesel::prelude::*;

pub struct OrderRepositoryImpl {
    pool: DbPool,
}

impl OrderRepositoryImpl {
    pub fn new(pool: DbPool) -> Self {
        Self { pool }
    }
}

impl OrderRepository for OrderRepositoryImpl {
    fn create_order(&self, order: &NewOrder) -> Result<Order> {
        let mut conn = self.pool.get()?;

        diesel::insert_into(orders::table)
            .values(order)
            .get_result(&mut conn)
            .map_err(Into::into)
    }

    fn get_order(&self, id: &str) -> Result<Option<Order>> {
        let mut conn = self.pool.get()?;

        orders::table
            .find(id)
            .first(&mut conn)
            .optional()
            .map_err(Into::into)
    }


}

// repository.rs
// Implementation of repository pattern for database operations

use super::db::{DbConnection, DbPool};
use super::models::*;
use super::provider::{DatabaseReader, DatabaseWriter};
use super::schema::*;
use anyhow::Result;
use bigdecimal::BigDecimal;
use diesel::prelude::*;

#[derive(Debug, Clone)]
pub struct Repository {
    pool: DbPool,
}

impl Repository {
    pub fn new(pool: DbPool) -> Self {
        Self { pool }
    }

    fn get_conn(&self) -> Result<DbConnection> {
        Ok(self.pool.get()?)
    }

    // Market operations
    pub fn create_market(&self, market_data: NewMarket) -> Result<Market> {
        let conn = &mut self.get_conn()?;

        let result = diesel::insert_into(markets::table)
            .values(&market_data)
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

    // Order operations
    pub fn create_order(&self, order_data: NewOrder) -> Result<Order> {
        let conn = &mut self.get_conn()?;

        let result = diesel::insert_into(orders::table)
            .values(&order_data)
            .get_result(conn)?;

        Ok(result)
    }

    pub fn get_order(&self, order_id: &str) -> Result<Option<Order>> {
        let conn = &mut self.get_conn()?;

        let result = orders::table.find(order_id).first(conn).optional()?;

        Ok(result)
    }

    pub fn update_order(
        &self,
        order_id: &str,
        remain: BigDecimal,
        frozen: BigDecimal,
        filled_base: BigDecimal,
        filled_quote: BigDecimal,
        filled_fee: BigDecimal,
        partially_filled: bool,
        status: &str,
    ) -> Result<Order> {
        let conn = &mut self.get_conn()?;

        let current_time = chrono::Utc::now().timestamp();

        let result = diesel::update(orders::table.find(order_id))
            .set((
                orders::remain.eq(remain),                
                orders::filled_base.eq(filled_base),
                orders::filled_quote.eq(filled_quote),
                orders::filled_fee.eq(filled_fee),                
                orders::status.eq(status),
                orders::update_time.eq(current_time),
            ))
            .get_result(conn)?;

        Ok(result)
    }

    pub fn get_open_orders_for_market(&self, market_id: &str) -> Result<Vec<Order>> {
        let conn = &mut self.get_conn()?;

        let result = orders::table
            .filter(orders::market_id.eq(market_id))
            .filter(orders::status.eq("OPEN"))
            .order(orders::create_time.desc())
            .load(conn)?;

        Ok(result)
    }

    pub fn get_user_orders(&self, user_id: &str, limit: i64) -> Result<Vec<Order>> {
        let conn = &mut self.get_conn()?;

        let result = orders::table
            .filter(orders::user_id.eq(user_id))
            .order(orders::create_time.desc())
            .limit(limit)
            .load(conn)?;

        Ok(result)
    }

    // Trade operations
    pub fn create_trade(&self, trade_data: NewTrade) -> Result<Trade> {
        let conn = &mut self.get_conn()?;

        let result = diesel::insert_into(trades::table)
            .values(&trade_data)
            .get_result(conn)?;

        Ok(result)
    }
    pub fn create_trades(&self, trades_data: Vec<NewTrade>) -> Result<Vec<Trade>> {
        let conn = &mut self.get_conn()?;

        let result = diesel::insert_into(trades::table)
            .values(&trades_data)
            .get_results(conn)?;

        Ok(result)
    }

    pub fn get_trades_for_market(&self, market_id: &str, limit: i64) -> Result<Vec<Trade>> {
        let conn = &mut self.get_conn()?;

        let result = trades::table
            .filter(trades::market_id.eq(market_id))
            .order(trades::timestamp.desc())
            .limit(limit)
            .load(conn)?;

        Ok(result)
    }

    pub fn get_trades_for_order(&self, order_id: &str) -> Result<Vec<Trade>> {
        let conn = &mut self.get_conn()?;

        let result = trades::table
            .filter(
                trades::taker_order_id
                    .eq(order_id)
                    .or(trades::maker_order_id.eq(order_id)),
            )
            .order(trades::timestamp.desc())
            .load(conn)?;

        Ok(result)
    }

    pub fn get_user_trades(&self, user_id: &str, limit: i64) -> Result<Vec<Trade>> {
        let conn = &mut self.get_conn()?;

        let result = trades::table
            .filter(
                trades::taker_user_id
                    .eq(user_id)
                    .or(trades::maker_user_id.eq(user_id)),
            )
            .order(trades::timestamp.desc())
            .limit(limit)
            .load(conn)?;

        Ok(result)
    }

    // Balance operations
    pub fn get_balance(&self, user_id: &str, asset: &str) -> Result<Option<Balance>> {
        let conn = &mut self.get_conn()?;

        let result = balances::table
            .find((user_id, asset))
            .first(conn)
            .optional()?;

        Ok(result)
    }

    pub fn update_or_create_balance(
        &self,
        user_id: &str,
        asset: &str,
        available_delta: BigDecimal,
        frozen_delta: BigDecimal,
    ) -> Result<Balance> {
        let conn = &mut self.get_conn()?;

        let current_time = chrono::Utc::now().timestamp();

        // Check if balance exists
        let balance_option = balances::table
            .find((user_id, asset))
            .first::<Balance>(conn)
            .optional()?;

        if let Some(balance) = balance_option {
            // Update existing balance
            let new_available = balance.available + available_delta.clone();
            let new_frozen = balance.frozen + frozen_delta.clone();

            // Ensure balances are not negative
            if new_available < BigDecimal::from(0) || new_frozen < BigDecimal::from(0) {
                return Err(anyhow::anyhow!("Insufficient balance"));
            }

            let result = diesel::update(balances::table.find((user_id, asset)))
                .set((
                    balances::available.eq(new_available),
                    balances::frozen.eq(new_frozen),
                    balances::update_time.eq(current_time),
                ))
                .get_result(conn)?;

            Ok(result)
        } else {
            // Create new balance
            let new_balance = NewBalance {
                user_id: user_id.to_string(),
                asset: asset.to_string(),
                available: available_delta,
                frozen: frozen_delta,
                update_time: current_time,
            };

            let result = diesel::insert_into(balances::table)
                .values(&new_balance)
                .get_result(conn)?;

            Ok(result)
        }
    }

    // Market stats operations
    pub fn update_market_stats(
        &self,
        market_id: &str,
        high_24h: BigDecimal,
        low_24h: BigDecimal,
        volume_24h: BigDecimal,
        price_change_24h: BigDecimal,
        last_price: BigDecimal,
    ) -> Result<MarketStat> {
        let conn = &mut self.get_conn()?;

        let current_time = chrono::Utc::now().timestamp();

        // Check if stats exist
        let stats_option = market_stats::table
            .find(market_id)
            .first::<MarketStat>(conn)
            .optional()?;

        if let Some(_) = stats_option {
            // Update existing stats
            let result = diesel::update(market_stats::table.find(market_id))
                .set((
                    market_stats::high_24h.eq(high_24h),
                    market_stats::low_24h.eq(low_24h),
                    market_stats::volume_24h.eq(volume_24h),
                    market_stats::price_change_24h.eq(price_change_24h),
                    market_stats::last_price.eq(last_price),
                    market_stats::last_update_time.eq(current_time),
                ))
                .get_result(conn)?;

            Ok(result)
        } else {
            // Create new stats
            let new_stats = NewMarketStat {
                market_id: market_id.to_string(),
                high_24h,
                low_24h,
                volume_24h,
                price_change_24h,
                last_price,
                last_update_time: current_time,
            };

            let result = diesel::insert_into(market_stats::table)
                .values(&new_stats)
                .get_result(conn)?;

            Ok(result)
        }
    }

    pub fn get_market_stats(&self, market_id: &str) -> Result<Option<MarketStat>> {
        let conn = &mut self.get_conn()?;

        let result = market_stats::table.find(market_id).first(conn).optional()?;

        Ok(result)
    }
}

impl DatabaseReader for Repository {
    fn get_market(&self, market_id: &str) -> Result<Option<Market>, anyhow::Error> {
        self.get_market(market_id)
    }

    fn list_markets(&self) -> Result<Vec<Market>> {
        self.list_markets()
    }

    fn get_order(&self, order_id: &str) -> Result<Option<Order>> {
        self.get_order(order_id)
    }

    fn get_open_orders_for_market(&self, market_id: &str) -> Result<Vec<Order>> {
        self.get_open_orders_for_market(market_id)
    }

    fn get_user_orders(&self, user_id: &str, limit: i64) -> Result<Vec<Order>> {
        self.get_user_orders(user_id, limit)
    }

    fn get_trades_for_market(&self, market_id: &str, limit: i64) -> Result<Vec<Trade>> {
        self.get_trades_for_market(market_id, limit)
    }

    fn get_trades_for_order(&self, order_id: &str) -> Result<Vec<Trade>> {
        self.get_trades_for_order(order_id)
    }

    fn get_user_trades(&self, user_id: &str, limit: i64) -> Result<Vec<Trade>> {
        self.get_user_trades(user_id, limit)
    }

    fn get_balance(&self, user_id: &str, asset: &str) -> Result<Option<Balance>> {
        self.get_balance(user_id, asset)
    }

    fn get_market_stats(&self, market_id: &str) -> Result<Option<MarketStat>> {
        self.get_market_stats(market_id)
    }
}

impl DatabaseWriter for Repository {
    fn create_market(&self, market_data: NewMarket) -> Result<Market> {
        self.create_market(market_data)
    }

    fn create_order(&self, order_data: NewOrder) -> Result<Order> {
        self.create_order(order_data)
    }

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
    ) -> Result<Order> {
        self.update_order(
            order_id,
            remain,
            frozen,
            filled_base,
            filled_quote,
            filled_fee,
            partially_filled,
            status,
        )
    }

    fn create_trade(&self, trade_data: NewTrade) -> Result<Trade> {
        self.create_trade(trade_data)
    }

    fn update_or_create_balance(
        &self,
        user_id: &str,
        asset: &str,
        available_delta: BigDecimal,
        frozen_delta: BigDecimal,
    ) -> Result<Balance> {
        self.update_or_create_balance(user_id, asset, available_delta, frozen_delta)
    }

    fn update_market_stats(
        &self,
        market_id: &str,
        high_24h: BigDecimal,
        low_24h: BigDecimal,
        volume_24h: BigDecimal,
        price_change_24h: BigDecimal,
        last_price: BigDecimal,
    ) -> Result<MarketStat> {
        self.update_market_stats(
            market_id,
            high_24h,
            low_24h,
            volume_24h,
            price_change_24h,
            last_price,
        )
    }
}

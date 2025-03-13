use super::*;
use crate::models::*;
use crate::schema::*;
use diesel::prelude::*;
use serde::{Deserialize, Serialize};

// Pagination cursor structures
#[derive(Debug, Serialize, Deserialize)]
pub struct OrderCursor {
    pub create_time: i64,
    pub order_id: String,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct TradeCursor {
    pub timestamp: i64,
    pub trade_id: String,
}

#[derive(Debug)]
pub struct PaginationOptions<T> {
    pub limit: i64,
    pub cursor: Option<T>,
}

pub struct QueryRepositoryImpl {
    pool: DbPool,
}

impl QueryRepositoryImpl {
    pub fn new(pool: DbPool) -> Self {
        Self { pool }
    }

    fn encode_cursor<T: Serialize>(cursor: &T) -> Result<String> {
        let json = serde_json::to_string(cursor)?;
        Ok(base64::Engine::encode(
            &base64::engine::general_purpose::STANDARD,
            json,
        ))
    }

    fn decode_cursor<T: for<'de> Deserialize<'de>>(encoded: &str) -> Result<T> {
        let json = base64::Engine::decode(&base64::engine::general_purpose::STANDARD, encoded)?;
        let json_str = String::from_utf8(json)?;
        Ok(serde_json::from_str(&json_str)?)
    }
}

impl QueryRepository for QueryRepositoryImpl {
    fn get_user_orders(
        &self,
        user_id: &str,
        pagination: PaginationOptions<OrderCursor>,
    ) -> Result<(Vec<Order>, Option<String>)> {
        let mut conn = self.pool.get()?;
        let mut query = orders::table
            .filter(orders::user_id.eq(user_id))
            .into_boxed();

        // Apply cursor-based pagination
        if let Some(cursor) = pagination.cursor {
            query = query.filter(
                orders::create_time
                    .lt(cursor.create_time)
                    .or(orders::create_time
                        .eq(cursor.create_time)
                        .and(orders::id.lt(cursor.order_id))),
            );
        }

        // Execute query with limit + 1 to determine if there are more results
        let limit = pagination.limit + 1;
        let mut orders = query
            .order_by((orders::create_time.desc(), orders::id.desc()))
            .limit(limit)
            .load::<Order>(&mut conn)?;

        // Check if there are more results
        let has_more = orders.len() > pagination.limit as usize;
        if has_more {
            orders.pop(); // Remove the extra item
        }

        // Generate next cursor if there are more results
        let next_cursor = if has_more {
            if let Some(last_order) = orders.last() {
                let cursor = OrderCursor {
                    create_time: last_order.create_time,
                    order_id: last_order.id.clone(),
                };
                Some(self.encode_cursor(&cursor)?)
            } else {
                None
            }
        } else {
            None
        };

        Ok((orders, next_cursor))
    }

    fn get_market_trades(
        &self,
        market_id: &str,
        pagination: PaginationOptions<TradeCursor>,
    ) -> Result<(Vec<Trade>, Option<String>)> {
        let mut conn = self.pool.get()?;
        let mut query = trades::table
            .filter(trades::market_id.eq(market_id))
            .into_boxed();

        // Apply cursor-based pagination
        if let Some(cursor) = pagination.cursor {
            query = query.filter(
                trades::timestamp.lt(cursor.timestamp).or(trades::timestamp
                    .eq(cursor.timestamp)
                    .and(trades::id.lt(cursor.trade_id))),
            );
        }

        // Execute query with limit + 1
        let limit = pagination.limit + 1;
        let mut trades = query
            .order_by((trades::timestamp.desc()))
            .limit(limit)
            .load::<Trade>(&mut conn)?;

        // Check if there are more results
        let has_more = trades.len() > pagination.limit as usize;
        if has_more {
            trades.pop();
        }

        // Generate next cursor
        let next_cursor = if has_more {
            if let Some(last_trade) = trades.last() {
                let cursor = TradeCursor {
                    timestamp: last_trade.timestamp,
                    trade_id: last_trade.id.clone(),
                };
                Some(self.encode_cursor(&cursor)?)
            } else {
                None
            }
        } else {
            None
        };

        Ok((trades, next_cursor))
    }

    fn get_user_balances(&self, user_id: &str) -> Result<Vec<Balance>> {
        let mut conn = self.pool.get()?;
        let balances = balances::table
            .filter(balances::user_id.eq(user_id))
            .load::<Balance>(&mut conn)?;
        Ok(balances)
    }
}


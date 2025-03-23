use super::Repository;
use crate::filters::OrderFilter;
use crate::models::models::*;
use crate::models::schema::*;
use crate::provider::*;
use anyhow::Context;
use anyhow::Result;
use common::db::pagination::*;
use common::utils;
use diesel::prelude::*;

impl Repository {
    fn get_order_total_count(&self, filter: OrderFilter) -> Result<i64> {
        let conn = &mut self.get_conn()?;
        let mut count_query = orders::table.into_boxed();
        if let Some(order_id) = filter.order_id {
            count_query = count_query.filter(orders::id.eq(order_id));
        }
        if let Some(market_id) = filter.market_id {
            count_query = count_query.filter(orders::market_id.eq(market_id));
        }
        if let Some(user_id) = filter.user_id {
            count_query = count_query.filter(orders::user_id.eq(user_id));
        }
        if let Some(status) = filter.status {
            count_query = count_query.filter(orders::status.eq(status));
        }
        if let Some(side) = filter.side {
            count_query = count_query.filter(orders::side.eq(side));
        }
        if let Some(order_type) = filter.order_type {
            count_query = count_query.filter(orders::order_type.eq(order_type));
        }

        // Get total count
        let total_count: i64 = count_query.select(diesel::dsl::count_star()).first(conn)?;
        Ok(total_count)
    }
}

impl OrderDatabaseReader for Repository {
    fn get_order(&self, order_id: &str) -> Result<Option<Order>> {
        let conn = &mut self.get_conn()?;
        let order = orders::table
            .find(order_id)
            .first::<Order>(conn)
            .context("Order not found")?;
        Ok(Some(order))
    }
    fn get_active_orders(&self, market_id: &str) -> Result<Vec<Order>> {
        let conn = &mut self.get_conn()?;
        let orders = orders::table
            .filter(orders::status.eq_any(&[
                OrderStatus::Open.as_str(),
                OrderStatus::PartiallyFilled.as_str(),
            ]))
            .load::<Order>(conn)
            .context("Failed to fetch active orders")?;
        Ok(orders)
    }

    fn list_orders(
        &self,
        filter: OrderFilter,
        pagination: Option<Pagination>,
    ) -> Result<Paginated<Order>> {
        let conn = &mut self.get_conn()?;
        let pagination = pagination.unwrap_or_default();

        // Build base query
        let mut query = orders::table.into_boxed();

        // Apply filters
        let cloned_filter = filter.clone();
        if let Some(order_id) = filter.order_id {
            query = query.filter(orders::id.eq(order_id));
        }
        if let Some(market_id) = filter.market_id {
            query = query.filter(orders::market_id.eq(market_id));
        }
        if let Some(user_id) = filter.user_id {
            query = query.filter(orders::user_id.eq(user_id));
        }
        if let Some(status) = filter.status {
            query = query.filter(orders::status.eq(status));
        }
        if let Some(side) = filter.side {
            query = query.filter(orders::side.eq(side));
        }
        if let Some(order_type) = filter.order_type {
            query = query.filter(orders::order_type.eq(order_type));
        }

        let limit = pagination.limit.unwrap_or(10);
        let offset = pagination.offset.unwrap_or(0);
        let total_count = self.get_order_total_count(cloned_filter)?;
        let mut orders = query
            .limit(limit + 1)
            .offset(offset)
            .load::<Order>(conn)
            .context("Failed to retrieve orders")?;

        // Check if there are more results
        let has_more = orders.len() > limit as usize;
        if has_more {
            orders.pop(); // Remove the extra item we fetched
        }

        let next_offset = if has_more { Some(offset + limit) } else { None };

        Ok(Paginated {
            items: orders,
            total_count,
            next_offset,
            has_more,
        })
    }
}

impl OrderDatabaseWriter for Repository {
    fn create_order(&self, order_data: NewOrder) -> Result<Order> {
        let conn = &mut self.get_conn()?;

        conn.transaction::<Order, anyhow::Error, _>(|conn| {
            // Get market details first
            let market = markets::table
                .find(&order_data.market_id)
                .first::<Market>(conn)
                .context("Failed to fetch market")?;

            // Calculate required amount based on order side
            let order_side = OrderSide::from_str(&order_data.side)
                .map_err(|e| anyhow::anyhow!("Invalid order side: {}", e))?;

            match order_side {
                OrderSide::Buy => {
                    // For buy orders, we need to lock quote_asset (price * amount)
                    let quote_amount = order_data.quote_amount.clone();

                    // Decrease available and increase frozen (freezing the funds)
                    self.lock_balance(&order_data.user_id, &market.quote_asset, quote_amount)
                        .context("Failed to update buyer balance")?;
                }
                OrderSide::Sell => {
                    // For sell orders, we need to lock base_asset
                    // Decrease available and increase frozen (freezing the funds)
                    self.lock_balance(
                        &order_data.user_id,
                        &market.base_asset,
                        order_data.base_amount.clone(),
                    )
                    .context("Failed to update seller balance")?;
                }
            }

            // Create the order
            let result = diesel::insert_into(orders::table)
                .values(&order_data)
                .get_result(conn)
                .unwrap();

            Ok(result)
        })
    }

    fn cancel_order(&self, order_id: &str) -> Result<Order> {
        let conn = &mut self.get_conn()?;
        conn.transaction::<Order, anyhow::Error, _>(|conn| {
            // Fetch the order first
            let order = orders::table
                .filter(orders::id.eq(order_id))
                .first::<Order>(conn)
                .context("Order not found")?;

            // Check if order is already in a final state
            let current_status = OrderStatus::from_str(&order.status)
                .map_err(|e| anyhow::anyhow!("Failed to parse order status: {}", e))?;
            if matches!(
                current_status,
                OrderStatus::Filled | OrderStatus::Canceled | OrderStatus::Rejected
            ) {
                return Err(anyhow::anyhow!("Order already in final state"));
            }

            // Parse the order side
            let order_side = OrderSide::from_str(&order.side)
                .map_err(|e| anyhow::anyhow!("Failed to parse order side: {}", e))?;

            // Fetch the market to determine assets
            let market = markets::table
                .filter(markets::id.eq(&order.market_id))
                .first::<Market>(conn)
                .context("Market not found")?;

            // Calculate remaining amount to unfreeze
            let (asset, unlock_amount) = match order_side {
                OrderSide::Buy => (market.quote_asset.clone(), order.remained_quote.clone()),
                OrderSide::Sell => (market.base_asset.clone(), order.remained_base.clone()),
            };

            // Update order status to CANCELED
            let updated_order = diesel::update(orders::table.find(order_id))
                .set((
                    orders::status.eq(OrderStatus::Canceled.as_str()),
                    orders::update_time.eq(utils::get_utc_now_millis()),
                ))
                .get_result::<Order>(conn)
                .context("Failed to update order status")?;

            // Unlock the balance
            diesel::update(wallets::table)
                .filter(wallets::user_id.eq(&order.user_id))
                .filter(wallets::asset.eq(&asset))
                .set((  
                    wallets::available.eq(wallets::available + unlock_amount.clone()),
                    wallets::locked.eq(wallets::locked - unlock_amount),
                ))
                .execute(conn)
                .context("Failed to unlock balance")?;

            Ok(updated_order)
        })
    }

    /// Cancel all active orders for a specific market
    fn cancel_all_orders(&self, market_id: &str) -> Result<Vec<Order>> {
        let conn = &mut self.get_conn()?;
        conn.transaction::<Vec<Order>, anyhow::Error, _>(|conn| {
            // Fetch all active orders for the market
            let active_orders = orders::table
                .filter(orders::market_id.eq(market_id))
                .filter(orders::status.eq_any(&[
                    OrderStatus::Open.as_str(),
                    OrderStatus::PartiallyFilled.as_str(),
                ]))
                .load::<Order>(conn)
                .context("Failed to fetch active orders")?;

            let mut canceled_orders = Vec::new();

            // Fetch market details
            let market = markets::table
                .filter(markets::id.eq(market_id))
                .first::<Market>(conn)
                .context("Market not found")?;

            for order in active_orders {
                // Parse the order side
                let order_side = OrderSide::from_str(&order.side)
                    .map_err(|e| anyhow::anyhow!("Invalid order side {}", e))?;

                // Determine the asset to unlock based on order side
                let (asset, unlock_amount) = match order_side {
                    OrderSide::Buy => (market.quote_asset.clone(), order.remained_quote.clone()),
                    OrderSide::Sell => (market.base_asset.clone(), order.remained_base.clone()),
                };

                // Update order status to CANCELED
                let canceled_order = diesel::update(orders::table.find(&order.id))
                    .set((
                        orders::status.eq(OrderStatus::Canceled.as_str()),
                        orders::update_time.eq(utils::get_utc_now_millis()),
                    ))
                    .get_result::<Order>(conn)
                    .context("Failed to update order status")?;

                // Unlock the balance
                diesel::update(wallets::table)
                    .filter(wallets::user_id.eq(&order.user_id))
                    .filter(wallets::asset.eq(&asset))
                    .set((
                        wallets::available.eq(wallets::available + unlock_amount.clone()),
                        wallets::locked.eq(wallets::locked - unlock_amount),
                    ))
                    .execute(conn)
                    .context("Failed to unlock balance")?;

                canceled_orders.push(canceled_order);
            }

            Ok(canceled_orders)
        })
    }

    /// Cancel all active orders globally
    fn cancel_all_global_orders(&self) -> Result<Vec<Order>> {
        let conn = &mut self.get_conn()?;
        conn.transaction::<Vec<Order>, anyhow::Error, _>(|conn| {
            // Fetch all active orders across all markets
            let active_orders = orders::table
                .filter(orders::status.eq_any(&[
                    OrderStatus::Open.as_str(),
                    OrderStatus::PartiallyFilled.as_str(),
                ]))
                .load::<Order>(conn)
                .context("Failed to fetch active orders")?;

            let mut canceled_orders = Vec::new();

            for order in active_orders {
                // Parse the order side
                let order_side = OrderSide::from_str(&order.side)
                    .map_err(|e| anyhow::anyhow!("Failed to parse order side: {}", e))?;

                // Fetch the market to determine assets
                let market = markets::table
                    .filter(markets::id.eq(&order.market_id))
                    .first::<Market>(conn)
                    .context("Market not found")?;

                // Determine the asset to unlock based on order side
                let (asset, unlock_amount) = match order_side {
                    OrderSide::Buy => (market.quote_asset.clone(), order.remained_quote.clone()),
                    OrderSide::Sell => (market.base_asset.clone(), order.remained_base.clone()),
                };

                // Update order status to CANCELED
                let canceled_order = diesel::update(orders::table.find(&order.id))
                    .set((
                        orders::status.eq(OrderStatus::Canceled.as_str()),
                        orders::update_time.eq(utils::get_utc_now_millis()),
                    ))
                    .get_result::<Order>(conn)
                    .context("Failed to update order status")?;

                // Unlock the balance
                diesel::update(wallets::table)
                    .filter(wallets::user_id.eq(&order.user_id))
                    .filter(wallets::asset.eq(&asset))
                    .set((
                        wallets::available.eq(wallets::available + unlock_amount.clone()),
                        wallets::locked.eq(wallets::locked - unlock_amount),
                    ))
                    .execute(conn)
                    .context("Failed to unlock balance")?;

                canceled_orders.push(canceled_order);
            }

            Ok(canceled_orders)
        })
    }

    fn update_order_status(&self, order_id: &str, status: OrderStatus) -> Result<Order> {
        let conn = &mut self.get_conn()?;
        let updated_order = diesel::update(orders::table.find(order_id))
            .set((orders::status.eq(status.as_str())))
            .get_result::<Order>(conn)
            .context("Failed to update order status")?;

        Ok(updated_order)
    }
}

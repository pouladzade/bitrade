use super::Repository;
use crate::models::models::*;
use crate::models::schema::*;
use anyhow::Context;
use anyhow::Result;
use bigdecimal::BigDecimal;
use diesel::prelude::*;

impl Repository {
    // Order operations
    pub fn create_order(&self, order_data: NewOrder) -> Result<Order> {
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

    pub fn get_order(&self, order_id: &str) -> Result<Option<Order>> {
        let conn = &mut self.get_conn()?;
        let result = orders::table.find(order_id).first(conn).optional()?;

        Ok(result)
    }

    pub fn get_open_orders_for_market(&self, market_id: &str) -> Result<Vec<Order>> {
        let conn = &mut self.get_conn()?;

        let result = orders::table
            .filter(orders::market_id.eq(market_id))
            .filter(orders::status.eq_any(&[
                OrderStatus::Open.as_str(),
                OrderStatus::PartiallyFilled.as_str(),
            ]))
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

    pub fn cancel_order(&self, order_id: &str) -> Result<Order> {
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
                    orders::update_time.eq(chrono::Utc::now().timestamp()),
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
    pub fn cancel_all_orders(&self, market_id: &str) -> Result<Vec<Order>> {
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
                        orders::update_time.eq(chrono::Utc::now().timestamp_millis()),
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
    pub fn cancel_all_global_orders(&self) -> Result<Vec<Order>> {
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
                        orders::update_time.eq(chrono::Utc::now().timestamp()),
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

    /// Recover active orders for a specific market to reload into the order book
    pub fn get_active_orders(&self, market_id: &str) -> Result<Vec<Order>> {
        let conn = &mut self.get_conn()?;
        orders::table
            .filter(orders::market_id.eq(market_id))
            .filter(orders::status.eq_any(&[
                OrderStatus::Open.as_str(),
                OrderStatus::PartiallyFilled.as_str(),
            ]))
            .filter(orders::remained_base.gt(BigDecimal::from(0)))
            .order(orders::create_time.asc())
            .load::<Order>(conn)
            .context("Failed to retrieve active orders")
    }

    /// Recover all active orders across all markets
    pub fn get_all_active_orders(&self) -> Result<Vec<Order>> {
        let conn = &mut self.get_conn()?;
        orders::table
            .filter(orders::status.eq_any(&[
                OrderStatus::Open.as_str(),
                OrderStatus::PartiallyFilled.as_str(),
            ]))
            .filter(orders::remained_base.gt(BigDecimal::from(0)))
            .order(orders::create_time.asc())
            .load::<Order>(conn)
            .context("Failed to retrieve all active orders")
    }

    pub fn get_user_active_orders_count(
        &self,
        market_id: &str,
        user_id: &str,
    ) -> Result<Vec<Order>> {
        let conn = &mut self.get_conn()?;
        orders::table
            .filter(orders::market_id.eq(market_id))
            .filter(orders::user_id.eq(user_id))
            .filter(orders::status.eq_any(&[
                OrderStatus::Open.as_str(),
                OrderStatus::PartiallyFilled.as_str(),
            ]))
            .order(orders::create_time.asc())
            .load::<Order>(conn)
            .context("Failed to retrieve all active orders")
    }

    pub fn update_order_status(&self, order_id: &str, status: OrderStatus) -> Result<Order> {
        let conn = &mut self.get_conn()?;
        let updated_order = diesel::update(orders::table.find(order_id))
            .set((orders::status.eq(status.as_str())))
            .get_result::<Order>(conn)
            .context("Failed to update order status")?;

        Ok(updated_order)
    }
}

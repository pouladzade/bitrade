// repository.rs
// Implementation of repository pattern for database operations

use crate::models::models::*;

use crate::models::schema::*;
use crate::{DbConnection, DbPool};
use anyhow::Context;
use anyhow::Result;
use bigdecimal::BigDecimal;
use chrono::Utc;
use diesel::prelude::*;
use uuid::Uuid;

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

    pub fn execute_limit_trade(
        &self,
        is_buyer_taker: bool,
        market_id: String,
        base_asset: String,
        quote_asset: String,
        buyer_user_id: String,
        seller_user_id: String,
        buyer_order_id: String,
        seller_order_id: String,
        price: BigDecimal,
        base_amount: BigDecimal,
        quote_amount: BigDecimal,
        buyer_fee_rate: BigDecimal,
        seller_fee_rate: BigDecimal,
    ) -> Result<NewTrade> {
        // Ensure buyer and seller are not the same user
        // if buyer_user_id == seller_user_id {
        //     return Err(anyhow::anyhow!("Buyer and seller cannot be the same user"));
        // }

        let conn = &mut self.get_conn()?;
        conn.transaction::<_, anyhow::Error, _>(|conn| {
            // 🔹 Fetch & Lock Seller's Balance
            let seller_base_balance: Balance = balances::table
                .filter(balances::user_id.eq(&seller_user_id))
                .filter(balances::asset.eq(&base_asset))
                .for_update()
                .first(conn)
                .context("Failed to fetch seller balance")?;

            let buyer_quote_balance: Balance = balances::table
                .filter(balances::user_id.eq(&buyer_user_id))
                .filter(balances::asset.eq(&quote_asset))
                .for_update()
                .first(conn)
                .context("Failed to fetch buyer balance")?;

            // 🔹 Ensure the seller has enough frozen balance
            if seller_base_balance.locked < base_amount {
                return Err(anyhow::anyhow!(
                    "Insufficient frozen balance: seller {} has {} {} frozen but needs {}",
                    seller_user_id,
                    seller_base_balance.locked,
                    base_asset,
                    base_amount
                ));
            }

            // 🔹 Ensure the buyer has enough frozen balance
            if buyer_quote_balance.locked < quote_amount {
                return Err(anyhow::anyhow!(
                    "Insufficient frozen balance: buyer {} has {} {} frozen but needs {}",
                    buyer_user_id,
                    buyer_quote_balance.locked,
                    quote_asset,
                    quote_amount
                ));
            }
            // 🔹 Calculate fees
            // buyer fee is calculated on the base amount (spent amount)
            let buyer_fee = (buyer_fee_rate * &base_amount).with_prec(8);
            // seller fee is calculated on the quote amount (received amount)
            let seller_fee = (seller_fee_rate * &quote_amount).with_prec(8);
            // 🔹 Fetch & Lock Seller Order
            let seller_order: Order = orders::table
                .filter(orders::id.eq(&seller_order_id))
                .filter(orders::status.eq_any(&[
                    OrderStatus::Open.as_str(),
                    OrderStatus::PartiallyFilled.as_str(),
                ]))
                .for_update()
                .first(conn)
                .context("Failed to fetch seller order")?;
            println!("seller_order.remained_base: {}", seller_order.remained_base);
            let new_seller_filled_base =
                &seller_order.filled_base.with_prec(8) + &base_amount.with_prec(8);
            let new_seller_filled_quote =
                &seller_order.filled_quote.with_prec(8) + &quote_amount.with_prec(8);
            let new_seller_filled_fee =
                (&seller_order.filled_fee.with_prec(8) + &seller_fee).with_prec(8);
            let new_seller_remained_base =
                &seller_order.remained_base.with_prec(8) - &base_amount.with_prec(8);
            // remained quote is not needed for the seller order
            // let new_seller_remained_quote =
            //     &seller_order.remained_quote.with_prec(8) - &quote_amount.with_prec(8);
            let seller_status =
                if new_seller_filled_base.with_prec(8) >= seller_order.base_amount.with_prec(8) {
                    OrderStatus::Filled.as_str()
                } else {
                    OrderStatus::PartiallyFilled.as_str()
                };

            // Debug printing for seller order calculations
            println!("Seller Order Update Values:");
            println!("  - Order ID: {}", seller_order_id);
            println!("  - Original filled_base: {}", seller_order.filled_base);
            println!("  - New filled_base: {}", new_seller_filled_base);
            println!("  - Original filled_quote: {}", seller_order.filled_quote);
            println!("  - New filled_quote: {}", new_seller_filled_quote);
            println!("  - Original filled_fee: {}", seller_order.filled_fee);
            println!("  - New filled_fee: {}", new_seller_filled_fee);
            println!("  - Original remained_base: {}", seller_order.remained_base);
            println!("  - New remained_base: {}", new_seller_remained_base);
            println!(
                "  - Original remained_quote: {}",
                seller_order.remained_quote
            );

            println!(
                "  - amount being traded: base={}, quote={}",
                base_amount, quote_amount
            );
            println!("  - fee: {}", seller_fee);
            println!("  - new status: {}", seller_status);

            diesel::update(&seller_order)
                .set((
                    orders::filled_base.eq(new_seller_filled_base.with_prec(8)),
                    orders::filled_quote.eq(new_seller_filled_quote.with_prec(8)),
                    orders::filled_fee.eq(new_seller_filled_fee.with_prec(8)),
                    orders::remained_base.eq(new_seller_remained_base.with_prec(8)),
                    orders::status.eq(seller_status),
                ))
                .execute(conn)
                .context("Failed to update seller order")?;

            // 🔹 Fetch & Lock Buyer Order
            let buyer_order: Order = orders::table
                .filter(orders::id.eq(&buyer_order_id))
                .filter(orders::status.eq_any(&[
                    OrderStatus::Open.as_str(),
                    OrderStatus::PartiallyFilled.as_str(),
                ]))
                .for_update()
                .first(conn)
                .context("Failed to fetch buyer order")?;

            let new_buyer_filled_base =
                &buyer_order.filled_base.with_prec(8) + &base_amount.with_prec(8);
            let new_buyer_filled_quote =
                &buyer_order.filled_quote.with_prec(8) + &quote_amount.with_prec(8);
            let new_buyer_filled_fee =
                (&buyer_order.filled_fee.with_prec(8) + &buyer_fee).with_prec(8);
            let new_buyer_remained_base =
                &buyer_order.remained_base.with_prec(8) - &base_amount.with_prec(8);
            let new_buyer_remained_quote =
                &buyer_order.remained_quote.with_prec(8) - &quote_amount.with_prec(8);

            // Debug printing for buyer order calculations
            println!("Buyer Order Update Values:");
            println!("  - Order ID: {}", buyer_order_id);
            println!("  - Original filled_base: {}", buyer_order.filled_base);
            println!("  - New filled_base: {}", new_buyer_filled_base);
            println!("  - Original filled_quote: {}", buyer_order.filled_quote);
            println!("  - New filled_quote: {}", new_buyer_filled_quote);
            println!("  - Original filled_fee: {}", buyer_order.filled_fee);
            println!("  - New filled_fee: {}", new_buyer_filled_fee);
            println!("  - Original remained_base: {}", buyer_order.remained_base);
            println!("  - New remained_base: {}", new_buyer_remained_base);
            println!(
                "  - Original remained_quote: {}",
                buyer_order.remained_quote
            );
            println!("  - New remained_quote: {}", new_buyer_remained_quote);
            println!(
                "  - amount being traded: base={}, quote={}",
                base_amount, quote_amount
            );
            println!("  - fee : {}", buyer_fee);

            let buyer_status =
                if new_buyer_filled_base.with_prec(8) >= buyer_order.base_amount.with_prec(8) {
                    OrderStatus::Filled.as_str()
                } else {
                    OrderStatus::PartiallyFilled.as_str()
                };

            diesel::update(&buyer_order)
                .set((
                    orders::filled_base.eq(&new_buyer_filled_base.with_prec(8)),
                    orders::filled_quote.eq(&new_buyer_filled_quote.with_prec(8)),
                    orders::filled_fee.eq(&new_buyer_filled_fee.with_prec(8)),
                    orders::remained_base.eq(&new_buyer_remained_base.with_prec(8)),
                    orders::remained_quote.eq(&new_buyer_remained_quote.with_prec(8)),
                    orders::status.eq(buyer_status),
                ))
                .execute(conn)
                .context("Failed to update buyer order")?;

            // 🔹 Calculate buyer's quote asset residue
            let buyer_quote_residue = if buyer_status == OrderStatus::Filled.as_str() {
                new_buyer_remained_quote
            } else {
                BigDecimal::from(0)
            };

            // 🔹 Deduct base asset from seller's frozen balance
            diesel::update(&seller_base_balance.clone())
                .set((balances::locked
                    .eq(seller_base_balance.locked.with_prec(8) - &base_amount.with_prec(8)),))
                .execute(conn)
                .context("Failed to update seller base balance")?;

            // 🔹 Deduct quote asset from buyer's frozen balance
            diesel::update(&buyer_quote_balance.clone())
                .set((
                    balances::locked.eq(buyer_quote_balance.locked.with_prec(8)
                        - &quote_amount.with_prec(8)
                        - &buyer_quote_residue.with_prec(8)),
                    balances::available.eq(buyer_quote_balance.available.with_prec(8)
                        + &buyer_quote_residue.with_prec(8)),
                ))
                .execute(conn)
                .context("Failed to update buyer quote balance")?;

            // 🔹 Fetch seller's quote balance to credit with quote amount
            let seller_quote_balance: Balance = balances::table
                .filter(balances::user_id.eq(&seller_user_id))
                .filter(balances::asset.eq(&quote_asset))
                .for_update()
                .first(conn)
                .context("Failed to fetch seller quote balance")?;

            // 🔹 Fetch buyer's base balance to credit with base amount
            let buyer_base_balance: Balance = balances::table
                .filter(balances::user_id.eq(&buyer_user_id))
                .filter(balances::asset.eq(&base_asset))
                .for_update()
                .first(conn)
                .context("Failed to fetch buyer base balance")?;

            let seller_receives = (&quote_amount - &seller_fee).with_prec(8);
            diesel::update(&seller_quote_balance.clone())
                .set(balances::available.eq(seller_quote_balance.available + seller_receives))
                .execute(conn)
                .context("Failed to update seller quote balance")?;

            let buyer_receives = (&base_amount - &buyer_fee).with_prec(8);
            diesel::update(&buyer_base_balance.clone())
                .set(balances::available.eq(buyer_base_balance.available + buyer_receives))
                .execute(conn)
                .context("Failed to update buyer base balance")?;
            // 🔹 Determine taker and maker for the trade record

            // 🔹 Update fee treasury for quote asset (seller fee)
            diesel::update(fee_treasury::table)
                .filter(fee_treasury::market_id.eq(&market_id))
                .filter(fee_treasury::asset.eq(&quote_asset))
                .set(
                    fee_treasury::collected_amount.eq(fee_treasury::collected_amount + &seller_fee),
                )
                .execute(conn)
                .context("Failed to update quote asset fee treasury")?;

            // 🔹 Update fee treasury for base asset (buyer fee)
            diesel::update(fee_treasury::table)
                .filter(fee_treasury::market_id.eq(&market_id))
                .filter(fee_treasury::asset.eq(&base_asset))
                .set(fee_treasury::collected_amount.eq(fee_treasury::collected_amount + &buyer_fee))
                .execute(conn)
                .context("Failed to update base asset fee treasury")?;
            // 🔹 Create and insert the trade record
            let new_trade = NewTrade {
                id: Uuid::new_v4().to_string(),
                timestamp: Utc::now().timestamp(),
                market_id,
                price,
                base_amount,
                quote_amount,
                buyer_user_id,
                buyer_order_id,
                buyer_fee,
                seller_user_id,
                seller_order_id,
                seller_fee,
                taker_side: if is_buyer_taker {
                    "BUY".to_string()
                } else {
                    "SELL".to_string()
                },
                is_liquidation: None,
            };

            diesel::insert_into(trades::table)
                .values(&new_trade)
                .execute(conn)
                .context("Failed to insert new trade")?;

            Ok(new_trade)
        })
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
                    self.update_or_create_balance(
                        &order_data.user_id,
                        &market.quote_asset,
                        -quote_amount.clone(),
                        quote_amount,
                    )
                    .context("Failed to update buyer balance")?;
                }
                OrderSide::Sell => {
                    // For sell orders, we need to lock base_asset
                    // Decrease available and increase frozen (freezing the funds)
                    self.update_or_create_balance(
                        &order_data.user_id,
                        &market.base_asset,
                        -order_data.base_amount.clone(),
                        order_data.base_amount.clone(),
                    )
                    .context("Failed to update seller balance")?;
                }
            }

            // Create the order
            let result = diesel::insert_into(orders::table)
                .values(&order_data)
                .get_result(conn)
                .context("Failed to insert order")?;

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
                trades::buyer_order_id
                    .eq(order_id)
                    .or(trades::seller_order_id.eq(order_id)),
            )
            .order(trades::timestamp.desc())
            .load(conn)?;

        Ok(result)
    }

    pub fn get_user_trades(&self, user_id: &str, limit: i64) -> Result<Vec<Trade>> {
        let conn = &mut self.get_conn()?;

        let result = trades::table
            .filter(
                trades::buyer_user_id
                    .eq(user_id)
                    .or(trades::seller_user_id.eq(user_id)),
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
        locked_delta: BigDecimal,
    ) -> Result<Balance> {
        let conn = &mut self.get_conn()?;

        let current_time = chrono::Utc::now().timestamp_millis();

        // Check if balance exists
        let balance_option = balances::table
            .find((user_id, asset))
            .first::<Balance>(conn)
            .optional()?;

        if let Some(balance) = balance_option {
            // Update existing balance
            let new_available = balance.available + available_delta.clone();
            let new_locked = balance.locked + locked_delta.clone();

            // Ensure balances are not negative
            if new_available < BigDecimal::from(0) || new_locked < BigDecimal::from(0) {
                return Err(anyhow::anyhow!("Insufficient balance"));
            }

            let result = diesel::update(balances::table.find((user_id, asset)))
                .set((
                    balances::available.eq(new_available),
                    balances::locked.eq(new_locked),
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
                locked: locked_delta,
                reserved: BigDecimal::from(0),
                total_deposited: BigDecimal::from(0),
                total_withdrawn: BigDecimal::from(0),
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

        let current_time = chrono::Utc::now().timestamp_millis();

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
            let (asset, unfreeze_amount) = match order_side {
                OrderSide::Buy => (
                    market.quote_asset.clone(),
                    order.quote_amount - order.filled_quote,
                ),
                OrderSide::Sell => (
                    market.base_asset.clone(),
                    order.base_amount - order.filled_base,
                ),
            };

            // Update order status to CANCELED
            let updated_order = diesel::update(orders::table.find(order_id))
                .set((
                    orders::status.eq(OrderStatus::Canceled.as_str()),
                    orders::update_time.eq(chrono::Utc::now().timestamp()),
                ))
                .get_result::<Order>(conn)
                .context("Failed to update order status")?;

            // Unfreeze the balance
            diesel::update(balances::table)
                .filter(balances::user_id.eq(&order.user_id))
                .filter(balances::asset.eq(&asset))
                .set((
                    balances::available.eq(balances::available + unfreeze_amount.clone()),
                    balances::locked.eq(balances::locked - unfreeze_amount),
                ))
                .execute(conn)
                .context("Failed to unfreeze balance")?;

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

                // Determine the asset to unfreeze based on order side
                let (asset, unfreeze_amount) = match order_side {
                    OrderSide::Buy => (
                        market.quote_asset.clone(),
                        order.quote_amount - order.filled_quote,
                    ),
                    OrderSide::Sell => (
                        market.base_asset.clone(),
                        order.base_amount - order.filled_base,
                    ),
                };

                // Update order status to CANCELED
                let canceled_order = diesel::update(orders::table.find(&order.id))
                    .set((
                        orders::status.eq(OrderStatus::Canceled.as_str()),
                        orders::update_time.eq(chrono::Utc::now().timestamp_millis()),
                    ))
                    .get_result::<Order>(conn)
                    .context("Failed to update order status")?;

                // Unfreeze the balance
                diesel::update(balances::table)
                    .filter(balances::user_id.eq(&order.user_id))
                    .filter(balances::asset.eq(&asset))
                    .set((
                        balances::available.eq(balances::available + unfreeze_amount.clone()),
                        balances::locked.eq(balances::locked - unfreeze_amount),
                    ))
                    .execute(conn)
                    .context("Failed to unfreeze balance")?;

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

                // Determine the asset to unfreeze based on order side
                let (asset, unfreeze_amount) = match order_side {
                    OrderSide::Buy => (
                        market.quote_asset.clone(),
                        order.quote_amount - order.filled_quote,
                    ),
                    OrderSide::Sell => (
                        market.base_asset.clone(),
                        order.base_amount - order.filled_base,
                    ),
                };

                // Update order status to CANCELED
                let canceled_order = diesel::update(orders::table.find(&order.id))
                    .set((
                        orders::status.eq(OrderStatus::Canceled.as_str()),
                        orders::update_time.eq(chrono::Utc::now().timestamp()),
                    ))
                    .get_result::<Order>(conn)
                    .context("Failed to update order status")?;

                // Unfreeze the balance
                diesel::update(balances::table)
                    .filter(balances::user_id.eq(&order.user_id))
                    .filter(balances::asset.eq(&asset))
                    .set((
                        balances::available.eq(balances::available + unfreeze_amount.clone()),
                        balances::locked.eq(balances::locked - unfreeze_amount),
                    ))
                    .execute(conn)
                    .context("Failed to unfreeze balance")?;

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
}

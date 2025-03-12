use super::Repository;
use crate::models::models::*;

use crate::models::schema::*;
use anyhow::Context;
use anyhow::Result;
use bigdecimal::BigDecimal;
use chrono::Utc;
use diesel::prelude::*;
use uuid::Uuid;

impl Repository {
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
        if buyer_user_id == seller_user_id {
            return Err(anyhow::anyhow!("Buyer and seller cannot be the same user"));
        }

        let conn = &mut self.get_conn()?;
        conn.transaction::<_, anyhow::Error, _>(|conn| {
            // 🔹 Fetch & Lock Seller's Balance
            let seller_base_balance: Wallet = wallets::table
                .filter(wallets::user_id.eq(&seller_user_id))
                .filter(wallets::asset.eq(&base_asset))
                .for_update()
                .first(conn)
                .context("Failed to fetch seller balance")?;

            let buyer_quote_balance: Wallet = wallets::table
                .filter(wallets::user_id.eq(&buyer_user_id))
                .filter(wallets::asset.eq(&quote_asset))
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

            diesel::update(orders::table)
                .filter(orders::id.eq(&seller_order_id))
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

            diesel::update(orders::table)
                .filter(orders::id.eq(&buyer_order_id))
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
            diesel::update(wallets::table)
                .filter(wallets::user_id.eq(&seller_user_id))
                .filter(wallets::asset.eq(&base_asset))
                .set((wallets::locked
                    .eq(seller_base_balance.locked.with_prec(8) - &base_amount.with_prec(8)),))
                .execute(conn)
                .context("Failed to update seller base balance")?;

            // 🔹 Deduct quote asset from buyer's frozen balance
            diesel::update(wallets::table)
                .filter(wallets::user_id.eq(&buyer_user_id))
                .filter(wallets::asset.eq(&quote_asset))
                .set((
                    wallets::locked.eq(buyer_quote_balance.locked.with_prec(8)
                        - &quote_amount.with_prec(8)
                        - &buyer_quote_residue.with_prec(8)),
                    wallets::available.eq(buyer_quote_balance.available.with_prec(8)
                        + &buyer_quote_residue.with_prec(8)),
                ))
                .execute(conn)
                .context("Failed to update buyer quote balance")?;

            // 🔹 Fetch seller's quote balance to credit with quote amount
            let seller_quote_balance: Wallet = wallets::table
                .filter(wallets::user_id.eq(&seller_user_id))
                .filter(wallets::asset.eq(&quote_asset))
                .for_update()
                .first(conn)
                .context("Failed to fetch seller quote balance")?;

            // 🔹 Fetch buyer's base balance to credit with base amount
            let buyer_base_balance: Wallet = wallets::table
                .filter(wallets::user_id.eq(&buyer_user_id))
                .filter(wallets::asset.eq(&base_asset))
                .for_update()
                .first(conn)
                .context("Failed to fetch buyer base balance")?;

            let seller_receives = (&quote_amount - &seller_fee).with_prec(8);
            diesel::update(wallets::table)
                .filter(wallets::user_id.eq(&seller_user_id))
                .filter(wallets::asset.eq(&quote_asset))
                .set(wallets::available.eq(seller_quote_balance.available + seller_receives))
                .execute(conn)
                .context("Failed to update seller quote balance")?;

            let buyer_receives = (&base_amount - &buyer_fee).with_prec(8);
            diesel::update(wallets::table)
                .filter(wallets::user_id.eq(&buyer_user_id))
                .filter(wallets::asset.eq(&base_asset))
                .set(wallets::available.eq(buyer_base_balance.available + buyer_receives))
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
                .unwrap();

            Ok(new_trade)
        })
    }
}

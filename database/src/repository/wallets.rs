use crate::filters::WalletFilter;
use crate::models::models::*;

use super::Repository;
use crate::models::schema::*;
use crate::provider::{WalletDatabaseReader, WalletDatabaseWriter};
use anyhow::{Result, bail};
use bigdecimal::BigDecimal;
use common::db::pagination::{Paginated, Pagination};
use diesel::prelude::*;

impl Repository {
    fn get_wallet_total_count(&self, filter: WalletFilter) -> Result<i64> {
        let conn = &mut self.get_conn()?;
        let mut count_query = wallets::table.into_boxed();
        if let Some(user_id) = filter.user_id {
            count_query = count_query.filter(wallets::user_id.eq(user_id));
        }
        if let Some(asset) = filter.asset {
            count_query = count_query.filter(wallets::asset.eq(asset));
        }

        // Get total count
        let total_count: i64 = count_query.select(diesel::dsl::count_star()).first(conn)?;
        Ok(total_count)
    }

    fn update_or_create_balance(
        &self,
        user_id: &str,
        asset: &str,
        available_delta: BigDecimal,
        locked_delta: BigDecimal,
    ) -> Result<Wallet> {
        let conn = &mut self.get_conn()?;
        let current_time = common::utils::get_utc_now_millis();

        let wallet_option = wallets::table
            .find((user_id, asset))
            .first::<Wallet>(conn)
            .optional()?;

        match wallet_option {
            Some(wallet) => {
                let new_available = wallet.available + available_delta.clone();
                let new_locked = wallet.locked + locked_delta.clone();

                if new_available < BigDecimal::from(0) || new_locked < BigDecimal::from(0) {
                    bail!("Insufficient balance");
                }

                let result = diesel::update(wallets::table.find((user_id, asset)))
                    .set((
                        wallets::available.eq(new_available),
                        wallets::locked.eq(new_locked),
                        wallets::update_time.eq(current_time),
                    ))
                    .get_result(conn)?;

                Ok(result)
            }
            None => {
                if available_delta < BigDecimal::from(0) || locked_delta < BigDecimal::from(0) {
                    bail!("Insufficient balance");
                }

                let new_wallet = NewWallet {
                    user_id: user_id.to_string(),
                    asset: asset.to_string(),
                    available: available_delta,
                    locked: locked_delta,
                    reserved: BigDecimal::from(0),
                    total_deposited: BigDecimal::from(0),
                    total_withdrawn: BigDecimal::from(0),
                    update_time: current_time,
                };

                let result = diesel::insert_into(wallets::table)
                    .values(&new_wallet)
                    .get_result(conn)?;

                Ok(result)
            }
        }
    }
}

impl WalletDatabaseReader for Repository {
    fn get_wallet(&self, user_id: &str, asset: &str) -> Result<Option<Wallet>> {
        let conn = &mut self.get_conn()?;

        let result = wallets::table
            .find((user_id, asset))
            .first(conn)
            .optional()?;

        Ok(result)
    }

    fn list_wallets(
        &self,
        filter: WalletFilter,
        pagination: Option<Pagination>,
    ) -> Result<Paginated<Wallet>> {
        let conn = &mut self.get_conn()?;
        let pagination = pagination.unwrap_or_default();
        let limit = pagination.limit.unwrap_or(10).min(100);
        let offset = pagination.offset.unwrap_or(0);
        let order_by = pagination
            .order_by
            .unwrap_or_else(|| "update_time".to_string());
        let order_direction = pagination
            .order_direction
            .unwrap_or_else(|| "desc".to_string());

        let mut query = wallets::table.into_boxed();
        let total: i64 = self.get_wallet_total_count(filter.clone())?;
        // Apply filters
        if let Some(user_id) = filter.user_id {
            query = query.filter(wallets::user_id.eq(user_id));
        }
        if let Some(asset) = filter.asset {
            query = query.filter(wallets::asset.eq(asset));
        }

        // Apply dynamic ordering with validation
        query = match (order_by.as_str(), order_direction.to_lowercase().as_str()) {
            ("update_time", "desc") => query.order(wallets::update_time.desc()),
            ("update_time", "asc") => query.order(wallets::update_time.asc()),
            ("asset", "desc") => query.order(wallets::asset.desc()),
            ("asset", "asc") => query.order(wallets::asset.asc()),
            ("user_id", "desc") => query.order(wallets::user_id.desc()),
            ("user_id", "asc") => query.order(wallets::user_id.asc()),
            (field, direction) => {
                // Invalid field or direction, return error
                bail!(
                    "Invalid order parameters: field '{}' or direction '{}'",
                    field,
                    direction
                );
            }
        };

        // Execute query with pagination

        let result = query.offset(offset).limit(limit).load::<Wallet>(conn)?;

        Ok(Paginated {
            items: result,
            total_count: total,
            next_offset: None,
            has_more: false,
        })
    }
}

impl WalletDatabaseWriter for Repository {
    fn lock_balance(&self, user_id: &str, asset: &str, amount: BigDecimal) -> Result<Wallet> {
        self.update_or_create_balance(user_id, asset, -amount.clone(), amount)
    }

    fn unlock_balance(&self, user_id: &str, asset: &str, amount: BigDecimal) -> Result<Wallet> {
        self.update_or_create_balance(user_id, asset, amount.clone(), amount)
    }

    fn deposit_balance(&self, user_id: &str, asset: &str, amount: BigDecimal) -> Result<Wallet> {
        let conn = &mut self.get_conn()?;
        let current_time = common::utils::get_utc_now_millis();

        let wallet = self.get_wallet(user_id, asset)?;

        match wallet {
            Some(wallet) => {
                let new_wallet = diesel::update(wallets::table.find((user_id, asset)))
                    .set((
                        wallets::available.eq(wallet.available + amount.clone()),
                        wallets::total_deposited.eq(wallet.total_deposited + amount.clone()),
                        wallets::update_time.eq(current_time),
                    ))
                    .get_result(conn)?;

                Ok(new_wallet)
            }
            None => {
                let new_wallet = NewWallet {
                    user_id: user_id.to_string(),
                    asset: asset.to_string(),
                    available: amount.clone(),
                    locked: BigDecimal::from(0),
                    reserved: BigDecimal::from(0),
                    total_deposited: amount.clone(),
                    total_withdrawn: BigDecimal::from(0),
                    update_time: current_time,
                };

                let result = diesel::insert_into(wallets::table)
                    .values(&new_wallet)
                    .get_result(conn)?;

                Ok(result)
            }
        }
    }

    fn withdraw_balance(&self, user_id: &str, asset: &str, amount: BigDecimal) -> Result<Wallet> {
        let conn = &mut self.get_conn()?;
        let current_time = common::utils::get_utc_now_millis();

        let balance = self.get_wallet(user_id, asset)?;

        match balance {
            Some(balance) => {
                if balance.available < amount {
                    bail!("Insufficient balance");
                }

                let new_balance = diesel::update(wallets::table.find((user_id, asset)))
                    .set((
                        wallets::available.eq(balance.available - amount.clone()),
                        wallets::total_withdrawn.eq(balance.total_withdrawn + amount.clone()),
                        wallets::update_time.eq(current_time),
                    ))
                    .get_result(conn)?;

                Ok(new_balance)
            }
            None => bail!("Balance not found"),
        }
    }
}

use crate::models::models::*;

use crate::models::schema::*;
use anyhow::{Result, bail};
use bigdecimal::BigDecimal;
use diesel::prelude::*;

use super::Repository;

impl Repository {
    fn get_current_timestamp(&self) -> i64 {
        chrono::Utc::now().timestamp_millis()
    }

    // Balance operations
    pub fn get_wallet(&self, user_id: &str, asset: &str) -> Result<Option<Wallet>> {
        let conn = &mut self.get_conn()?;

        let result = wallets::table
            .find((user_id, asset))
            .first(conn)
            .optional()?;

        Ok(result)
    }
    pub fn lock_balance(&self, user_id: &str, asset: &str, amount: BigDecimal) -> Result<Wallet> {
        self.update_or_create_balance(user_id, asset, -amount.clone(), amount)
    }

    pub fn unlock_balance(&self, user_id: &str, asset: &str, amount: BigDecimal) -> Result<Wallet> {
        self.update_or_create_balance(user_id, asset, amount.clone(), amount)
    }

    fn update_or_create_balance(
        &self,
        user_id: &str,
        asset: &str,
        available_delta: BigDecimal,
        locked_delta: BigDecimal,
    ) -> Result<Wallet> {
        let conn = &mut self.get_conn()?;
        let current_time = self.get_current_timestamp();

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

    pub fn deposit_balance(
        &self,
        user_id: &str,
        asset: &str,
        amount: BigDecimal,
    ) -> Result<Wallet> {
        let conn = &mut self.get_conn()?;
        let current_time = self.get_current_timestamp();

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

    pub fn withdraw_balance(
        &self,
        user_id: &str,
        asset: &str,
        amount: BigDecimal,
    ) -> Result<Wallet> {
        let conn = &mut self.get_conn()?;
        let current_time = self.get_current_timestamp();

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

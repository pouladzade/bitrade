use anyhow::{Context, Result};
use bigdecimal::BigDecimal;

use database::{models::models::Wallet, provider::DatabaseProvider};
use std::sync::Arc;

#[derive(Debug, Clone)]
pub struct WalletService<P: DatabaseProvider> {
    persister: Arc<P>,
}

impl<P: DatabaseProvider> WalletService<P> {
    /// Create a new wallet for a specific user
    pub fn new(persister: Arc<P>) -> Self {
        Self { persister }
    }

    /// Get the current balance for a specific asset
    pub fn get_balance(&self, asset: &str, user_id: &str) -> Result<BigDecimal> {
        let balance = self
            .persister
            .get_wallet(&user_id, asset)
            .context("Failed to retrieve balance")?
            .map(|b| b.available)
            .unwrap_or_else(|| BigDecimal::from(0));

        Ok(balance)
    }

    /// Get the frozen balance for a specific asset
    pub fn get_frozen_balance(&self, asset: &str, user_id: &str) -> Result<BigDecimal> {
        let balance = self
            .persister
            .get_wallet(&user_id, asset)
            .context("Failed to retrieve balance")?
            .map(|b| b.locked)
            .unwrap_or_else(|| BigDecimal::from(0));

        Ok(balance)
    }

    /// Add balance to a specific asset
    pub fn deposit(&self, asset: &str, amount: BigDecimal, user_id: &str) -> Result<Wallet> {
        if amount <= BigDecimal::from(0) {
            return Err(anyhow::anyhow!("Cannot add non-positive balance"));
        }

        self.persister
            .deposit_balance(&user_id, asset, amount.clone())
            .context("Failed to add balance")
    }

    /// Freeze balance for a specific asset
    pub fn lock_balance(&self, asset: &str, amount: BigDecimal, user_id: &str) -> Result<Wallet> {
        if amount <= BigDecimal::from(0) {
            return Err(anyhow::anyhow!("Cannot freeze non-positive balance"));
        }

        self.persister
            .lock_balance(&user_id, asset, amount)
            .context("Failed to freeze balance")
    }

    /// Unfreeze balance for a specific asset
    pub fn unlock_balance(&self, asset: &str, amount: BigDecimal, user_id: &str) -> Result<Wallet> {
        if amount <= BigDecimal::from(0) {
            return Err(anyhow::anyhow!("Cannot unfreeze non-positive balance"));
        }

        self.persister
            .unlock_balance(&user_id, asset, amount.clone())
            .context("Failed to unfreeze balance")
    }

    /// Withdraw balance from a specific asset
    pub fn withdraw(&self, asset: &str, amount: BigDecimal, user_id: &str) -> Result<Wallet> {
        if amount <= BigDecimal::from(0) {
            return Err(anyhow::anyhow!("Cannot withdraw non-positive amount"));
        }

        self.persister
            .withdraw_balance(
                &user_id,
                asset,
                amount.clone(), // Reduce available
            )
            .context("Failed to withdraw balance")
    }

    /// Get all balances for the user
    pub fn get_all_balances(&self) -> Result<Vec<Wallet>> {
        // Note: This method assumes you might want to add a method to Repository
        // to fetch all balances for a user if it doesn't already exist
        unimplemented!("Implement method to fetch all balances for a user")
    }
}

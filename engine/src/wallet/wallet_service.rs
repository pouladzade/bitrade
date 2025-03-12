use anyhow::{Context, Result};
use bigdecimal::BigDecimal;
use database::persistence::postgres_persister::PostgresPersister;

use database::models::models::Wallet;
use database::persistence::Persistence;

#[derive(Debug, Clone)]
pub struct WalletService {
    persister: PostgresPersister,
}

impl WalletService {
    /// Create a new wallet for a specific user
    pub fn new() -> Self {
        let database_url = "postgres://postgres:mysecretpassword@localhost/postgres";
        let pool_size = 10;
        let persister = PostgresPersister::new(database_url.to_string(), pool_size);
        Self { persister }
    }

    /// Get the current balance for a specific asset
    pub fn get_balance(&self, asset: &str, user_id: &str) -> Result<BigDecimal> {
        let balance = self
            .persister
            .get_balance(&user_id, asset)
            .context("Failed to retrieve balance")?
            .map(|b| b.available)
            .unwrap_or_else(|| BigDecimal::from(0));

        Ok(balance)
    }

    /// Get the frozen balance for a specific asset
    pub fn get_frozen_balance(&self, asset: &str, user_id: &str) -> Result<BigDecimal> {
        let balance = self
            .persister
            .get_balance(&user_id, asset)
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

// Example usage
// #[cfg(test)]
// mod tests {
//     use std::ptr::eq;

//     use super::*;
//     use chrono::Utc;
//     use mockall::mock;
//     use mockall::predicate::*;

//     mock! {
//         PersistenceMock {}
//         impl Persistence for PersistenceMock {
//             // Mock methods required for testing
//             fn update_or_create_balance(
//                 &self,
//                 user_id: &str,
//                 asset: &str,
//                 available_delta: BigDecimal,
//                 locked_delta: BigDecimal
//             ) -> Result<Balance>;

//             fn get_balance(&self, user_id: &str, asset: &str) -> Result<Option<Balance>>;
//         }
//     }

//     #[test]
//     fn test_add_balance() {
//         let mut mock_persistence = MockPersistenceMock::new();
//         mock_persistence
//             .expect_update_or_create_balance()
//             .with(
//                 eq("user123"),
//                 eq("BTC"),
//                 eq(BigDecimal::from(10)),
//                 eq(BigDecimal::from(0)),
//             )
//             .times(1)
//             .returning(|_, _, _, _| {
//                 Ok(Balance {
//                     user_id: "user123".to_string(),
//                     asset: "BTC".to_string(),
//                     available: BigDecimal::from(10),
//                     frozen: BigDecimal::from(0),
//                     update_time: Utc::now().timestamp_millis(),
//                 })
//             });

//         let wallet = Wallet::new("user123".to_string(), Arc::new(mock_persistence));

//         let result = wallet.add_balance("BTC", BigDecimal::from(10));
//         assert!(result.is_ok());
//     }
// }

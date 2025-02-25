use super::market::Market;
use crate::models::order::Order;
use crate::models::trade::Trade;
use anyhow::{anyhow, Context, Result};
use std::collections::HashMap;
use std::sync::{Arc, RwLock};
use tracing::field::debug;

#[derive(Debug, Clone)]
pub struct MarketManager {
    markets: Arc<RwLock<HashMap<String, Market>>>,
}

impl MarketManager {
    pub fn new() -> Self {
        MarketManager {
            markets: Arc::new(RwLock::new(HashMap::new())),
        }
    }

    pub fn create_market(&self, market_id: &str, pool_size: usize) -> Result<()> {
        let mut markets = self
            .markets
            .write()
            .map_err(|e| anyhow!("Failed to acquire write lock on markets: {}", e))?;

        if !markets.contains_key(market_id) {
            let market = Market::new(market_id.to_string(), pool_size);
            markets.insert(market_id.to_string(), market);
        }
        tracing::debug!(target: "market_manager", "Created market {}", market_id);
        println!("Created market {:?}", markets);
        Ok(())
    }

    pub fn start_market(&self, market_id: &str) -> Result<()> {
        let mut markets = self
            .markets
            .write()
            .map_err(|e| anyhow!("Failed to acquire write lock on markets: {}", e))?;

        let market = markets
            .get_mut(market_id)
            .context(format!("Market {} not found", market_id))?;
        market.start_market();
        tracing::debug!(target: "market_manager", "Started market {}", market_id);
        Ok(())
    }

    pub fn stop_market(&self, market_id: &str) -> Result<()> {
        let mut markets = self
            .markets
            .write()
            .map_err(|e| anyhow!("Failed to acquire write lock on markets: {}", e))?;

        let market = markets
            .get_mut(market_id)
            .context(format!("Market {} not found", market_id))?;
        market.stop_market();
        Ok(())
    }

    pub fn add_order(&self, order: Order) -> Result<(Vec<Trade>, String)> {
        let markets = self
            .markets
            .read()
            .map_err(|e| anyhow!("Failed to acquire read lock on markets: {}", e))?;

        let market = markets
            .get(&order.market_id)
            .context(format!("Market {} not found", order.market_id))?;

        market.add_order(order) // `Market::add_order` already uses tasks, so it's safe
    }

    pub fn cancel_order(&self, market_id: &str, order_id: String) -> Result<bool> {
        let markets = self
            .markets
            .read()
            .map_err(|e| anyhow!("Failed to acquire read lock on markets: {}", e))?;

        let market = markets
            .get(market_id)
            .context(format!("Market {} not found", market_id))?;
        market.cancel_order(order_id)
    }

    pub fn get_order_by_id(&self, market_id: &str, order_id: String) -> Result<Option<Order>> {
        let markets = self
            .markets
            .read()
            .map_err(|e| anyhow!("Failed to acquire read lock on markets: {}", e))?;

        let market = markets
            .get(market_id)
            .context(format!("Market {} not found", market_id))?;
        market.get_order_by_id(order_id)
    }

    pub fn cancel_all_orders(&self, market_id: &str) -> Result<bool> {
        let markets = self
            .markets
            .read()
            .map_err(|e| anyhow!("Failed to acquire read lock on markets: {}", e))?;

        let market = markets
            .get(market_id)
            .context(format!("Market {} not found", market_id))?;
        market.cancel_all_orders()
    }

    pub fn cancel_all_orders_global(&self) -> Result<()> {
        let markets = self
            .markets
            .read()
            .map_err(|e| anyhow!("Failed to acquire read lock on markets: {}", e))?;

        for market in markets.values() {
            market.cancel_all_orders()?;
        }
        Ok(())
    }
}

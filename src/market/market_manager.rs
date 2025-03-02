use super::market::Market;
use crate::models::trade_order::TradeOrder;
use crate::models::matched_trade::MatchedTrade;
use anyhow::{anyhow, Context, Result};
use std::collections::HashMap;
use std::sync::Arc;
use std::sync::RwLock;

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

    pub fn add_order(&self, order: TradeOrder) -> Result<(Vec<MatchedTrade>, String)> {
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

    pub fn get_order_by_id(&self, market_id: &str, order_id: String) -> Result<Option<TradeOrder>> {
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
#[cfg(test)]
mod tests {
    use crate::{
        models::trade_order::{OrderSide, OrderType},
        tests::test_models,
    };

    use super::*;
    const MARKET_ID: &str = "market_id";
    #[test]
    fn test_create_market() {
        let manager = MarketManager::new();
        assert!(manager.create_market(MARKET_ID, 10).is_ok());
        let markets = manager.markets.read().unwrap();
        assert!(markets.contains_key(MARKET_ID));
    }

    #[test]
    fn test_start_market() {
        let manager = MarketManager::new();
        manager.create_market(MARKET_ID, 10).unwrap();
        assert!(manager.start_market(MARKET_ID).is_ok());
    }

    #[test]
    fn test_stop_market() {
        let manager = MarketManager::new();
        manager.create_market(MARKET_ID, 10).unwrap();
        manager.start_market(MARKET_ID).unwrap();
        assert!(manager.stop_market(MARKET_ID).is_ok());
    }

    #[test]
    fn test_add_order() {
        let manager = MarketManager::new();

        manager.create_market(MARKET_ID, 10).unwrap();
        let order =
            test_models::create_order(OrderSide::Buy, "100", "10", OrderType::Limit, MARKET_ID);
        let result = manager.add_order(order.clone());
        assert!(result.is_err());
        manager.start_market(MARKET_ID).unwrap();
        let result = manager.add_order(order);
        assert!(result.is_ok());
    }

    #[test]
    fn test_cancel_order() {
        let manager = MarketManager::new();

        manager.create_market(MARKET_ID, 10).unwrap();
        manager.start_market(MARKET_ID).unwrap();
        let order =
            test_models::create_order(OrderSide::Buy, "100", "10", OrderType::Limit, MARKET_ID);
        let order_id = order.id.clone();
        manager.add_order(order).unwrap();
        assert!(manager.cancel_order(MARKET_ID, order_id).unwrap());
    }

    #[test]
    fn test_get_order_by_id() {
        let manager = MarketManager::new();
        manager.create_market(MARKET_ID, 10).unwrap();
        manager.start_market(MARKET_ID).unwrap();
        let order =
            test_models::create_order(OrderSide::Buy, "100", "10", OrderType::Limit, MARKET_ID);
        let order_id = order.id.clone();
        manager.add_order(order.clone()).unwrap();
        let fetched_order = manager.get_order_by_id(MARKET_ID, order_id).unwrap();
        assert_eq!(fetched_order, Some(order));
    }

    #[test]
    fn test_cancel_all_orders() {
        let manager = MarketManager::new();
        manager.create_market(MARKET_ID, 10).unwrap();
        manager.start_market(MARKET_ID).unwrap();
        let order1 =
            test_models::create_order(OrderSide::Buy, "100", "10", OrderType::Limit, MARKET_ID);
        let order2 =
            test_models::create_order(OrderSide::Buy, "200", "20", OrderType::Limit, MARKET_ID);
        manager.add_order(order1).unwrap();
        manager.add_order(order2).unwrap();
        assert!(manager.cancel_all_orders(MARKET_ID).unwrap());
    }

    #[test]
    fn test_cancel_all_orders_global() {
        let manager = MarketManager::new();
        let market_id1 = "test_market1";
        let market_id2 = "test_market2";
        manager.create_market(market_id1, 10).unwrap();
        manager.start_market(market_id1).unwrap();
        manager.create_market(market_id2, 10).unwrap();
        manager.start_market(market_id2).unwrap();
        let order1 =
            test_models::create_order(OrderSide::Buy, "100", "10", OrderType::Limit, market_id1);
        let order2 =
            test_models::create_order(OrderSide::Buy, "200", "20", OrderType::Limit, market_id2);
        manager.add_order(order1).unwrap();
        manager.add_order(order2).unwrap();
        assert!(manager.cancel_all_orders_global().is_ok());
    }
}

use super::market::Market;
use crate::models::matched_trade::MatchedTrade;
use crate::models::trade_order::TradeOrder;
use crate::utils;
use anyhow::{anyhow, Context, Result};
use bigdecimal::BigDecimal;
use database::models::models::NewMarket;
use database::persistence::persistence::Persistence;
use tonic::Status;
use std::collections::HashMap;
use std::str::FromStr;
use std::sync::{Arc, Mutex};
use std::thread::{self, JoinHandle};

#[derive(Debug)]
pub struct MarketManager<P>
where
    P: Persistence + 'static,
{
    markets: Arc<Mutex<HashMap<String, Arc<Mutex<Market<P>>>>>>,
    market_handles: Arc<Mutex<Vec<JoinHandle<()>>>>,
    persister: Arc<P>,
}

impl<P: Persistence> MarketManager<P> {
    pub fn new(persister: Arc<P>) -> Self {
        MarketManager {
            markets: Arc::new(Mutex::new(HashMap::new())),
            market_handles: Arc::new(Mutex::new(Vec::new())),
            persister,
        }
    }

    fn get_market(&self, market_id: &str) -> Result<Arc<Mutex<Market<P>>>> {
        let markets = self
            .markets
            .lock()
            .map_err(|e| anyhow!("Failed to acquire lock on markets: {}", e))?;

        markets
            .get(market_id)
            .cloned()
            .context(format!("Market {} not found", market_id))
    }

    pub fn create_market(
        &self,
        market_id: String,
        base_asset: String,
        quote_asset: String,
        default_maker_fee: String,
        default_taker_fee: String,
    ) -> Result<()> {
        let mut markets = self
            .markets
            .lock()
            .map_err(|e| anyhow!("Failed to acquire lock on markets: {}", e))?;

        if !markets.contains_key(market_id.as_str()) {
            let market = Arc::new(Mutex::new(Market::new(
                self.persister.clone(),
                market_id.to_string(),
            )?));
            markets.insert(market_id.to_string(), market);
            self.persister
                .create_market(NewMarket {
                    id: market_id.clone(),
                    base_asset: base_asset.clone(),
                    quote_asset: quote_asset.clone(),
                    default_maker_fee: BigDecimal::from_str(&default_maker_fee)
                        .context("Failed to parse amount as Decimal")
                        .map_err(|e| Status::invalid_argument(e.to_string()))?,
                    default_taker_fee: BigDecimal::from_str(&default_taker_fee)
                        .context("Failed to parse amount as Decimal")
                        .map_err(|e| Status::invalid_argument(e.to_string()))?,
                    create_time: utils::get_utc_now_time_millisecond(),
                    update_time: utils::get_utc_now_time_millisecond(),
                })
                .context("Failed to persist market")
                .map_err(|e| Status::internal(e.to_string()))?;
        }
        tracing::debug!(target: "market_manager", "Created market {}", market_id);
        Ok(())
    }

    pub fn start_market(&self, market_id: &str) -> Result<()> {
        let market = self.get_market(market_id)?;

        // Spawn a dedicated thread for this market
        let market_clone = Arc::clone(&market);
        let handle = thread::spawn(move || {
            let market = market_clone.lock().expect("Failed to lock market");
            market.start_market();
        });

        // Store the thread handle
        let mut handles = self
            .market_handles
            .lock()
            .map_err(|e| anyhow!("Failed to acquire lock on market handles: {}", e))?;
        handles.push(handle);

        tracing::debug!(target: "market_manager", "Started market {}", market_id);
        Ok(())
    }

    pub fn stop_market(&self, market_id: &str) -> Result<()> {
        let market = self.get_market(market_id)?;

        let mut market_guard = market
            .lock()
            .map_err(|e| anyhow!("Failed to lock market: {}", e))?;

        market_guard.stop_market();
        Ok(())
    }

    pub fn add_order(&self, order: TradeOrder) -> Result<(Vec<MatchedTrade>, String)> {
        let market = self.get_market(&order.market_id)?;

        let mut market_guard = market
            .lock()
            .map_err(|e| anyhow!("Failed to lock market: {}", e))?;

        let trade = market_guard.add_order(order)?;
        Ok((trade, market_guard.get_market_id()))
    }

    pub fn cancel_order(&self, market_id: &str, order_id: String) -> Result<bool> {
        let market = self.get_market(market_id)?;

        let mut market_guard = market
            .lock()
            .map_err(|e| anyhow!("Failed to lock market: {}", e))?;

        market_guard.cancel_order(order_id)
    }

    pub fn get_order_by_id(&self, market_id: &str, order_id: String) -> Result<TradeOrder> {
        let market = self.get_market(market_id)?;

        let market_guard = market
            .lock()
            .map_err(|e| anyhow!("Failed to lock market: {}", e))?;

        market_guard.get_order_by_id(order_id)
    }

    pub fn cancel_all_orders(&self, market_id: &str) -> Result<bool> {
        let market = self.get_market(market_id)?;

        let mut market_guard = market
            .lock()
            .map_err(|e| anyhow!("Failed to lock market: {}", e))?;

        market_guard.cancel_all_orders()
    }

    pub fn cancel_all_orders_global(&self) -> Result<()> {
        let markets = self
            .markets
            .lock()
            .map_err(|e| anyhow!("Failed to acquire lock on markets: {}", e))?;

        for market in markets.values() {
            let mut market_guard = market
                .lock()
                .map_err(|e| anyhow!("Failed to lock market: {}", e))?;
            market_guard.cancel_all_orders()?;
        }
        Ok(())
    }

    // Optional: Method to gracefully shutdown all market threads
    pub fn shutdown(&self) -> Result<()> {
        // Stop all markets first
        self.cancel_all_orders_global()?;

        // Join all market threads
        let mut handles = self
            .market_handles
            .lock()
            .map_err(|e| anyhow!("Failed to acquire lock on market handles: {}", e))?;

        for handle in handles.drain(..) {
            handle
                .join()
                .map_err(|_| anyhow!("Failed to join market thread"))?;
        }

        Ok(())
    }
}

// Implement Drop trait for clean thread termination
impl<P: Persistence> Drop for MarketManager<P> {
    fn drop(&mut self) {
        if let Ok(()) = self.shutdown() {
            tracing::debug!(target: "market_manager", "Gracefully shutdown all markets");
        }
    }
}

use super::market::Market;
use crate::models::matched_trade::MatchedTrade;
use crate::models::trade_order::TradeOrder;
use crate::utils;
use anyhow::{anyhow, Context, Result};
use bigdecimal::BigDecimal;
use database::models::models::{MarketStatus, NewMarket};
use database::persistence::persistence::Persistence;
use std::collections::HashMap;
use std::str::FromStr;
use std::sync::{Arc, Mutex};
use std::thread::{self, JoinHandle};
use tonic::Status;

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
        let manager = MarketManager {
            markets: Arc::new(Mutex::new(HashMap::new())),
            market_handles: Arc::new(Mutex::new(Vec::new())),
            persister: persister.clone(),
        };

        manager.load_markets_from_db();

        println!(
            "market_manager : Loaded {} markets from database",
            manager.markets.lock().unwrap().len()
        );
        manager
    }

    fn load_markets_from_db(&self) {
        // Load existing markets from database
        if let Ok(db_markets) = self.persister.list_markets() {
            for db_market in db_markets {
                println!(
                    "Loading market: id={}, base={}, quote={}",
                    db_market.id, db_market.base_asset, db_market.quote_asset
                );

                let market = Arc::new(Mutex::new(
                    Market::new(
                        self.persister.clone(),
                        db_market.id.clone(),
                        db_market.base_asset,
                        db_market.quote_asset,
                    )
                    .expect("Failed to create market"),
                ));

                if let Ok(mut markets) = self.markets.lock() {
                    markets.insert(db_market.id, market);
                }
            }
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
                base_asset.clone(),
                quote_asset.clone(),
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
                    amount_precision: 8,
                    min_base_amount: BigDecimal::from_str("0.00000000")
                        .context("Failed to parse amount as Decimal")
                        .map_err(|e| Status::invalid_argument(e.to_string()))?,
                    min_quote_amount: BigDecimal::from_str("0.00000000")
                        .context("Failed to parse amount as Decimal")
                        .map_err(|e| Status::invalid_argument(e.to_string()))?,
                    price_precision: 8,
                    status: MarketStatus::Active.as_str().to_string(),
                })
                .context("Failed to persist market")
                .map_err(|e| Status::internal(e.to_string()))?;
        }
        println!("market_manager : Created market {}", market_id);
        Ok(())
    }

    pub fn start_market(&self, market_id: &str) -> Result<()> {
        let market = self.get_market(market_id)?;

        // Spawn a dedicated thread for this market
        let market_clone = Arc::clone(&market);
        let handle = thread::spawn(move || {
            let market = market_clone.lock().expect("Failed to lock market");
            let _ = market.start_market();
        });

        // Store the thread handle
        let mut handles = self
            .market_handles
            .lock()
            .map_err(|e| anyhow!("Failed to acquire lock on market handles: {}", e))?;
        handles.push(handle);

        println!("market_manager : Started market {}", market_id);
        Ok(())
    }

    pub fn stop_market(&self, market_id: &str) -> Result<()> {
        let market = self.get_market(market_id)?;

        let market_guard = market
            .lock()
            .map_err(|e| anyhow!("Failed to lock market: {}", e))?;

        let _ = market_guard.stop_market();
        println!("market_manager : Stopped market {}", market_id);
        Ok(())
    }

    pub fn add_order(&self, order: TradeOrder) -> Result<(Vec<MatchedTrade>, String)> {
        let market = self.get_market(&order.market_id)?;

        let market_guard = market
            .lock()
            .map_err(|e| anyhow!("Failed to lock market: {}", e))?;

        let trade = market_guard.add_order(order)?;
        Ok((trade, market_guard.get_market_id()))
    }

    pub fn cancel_order(&self, market_id: &str, order_id: String) -> Result<bool> {
        let market = self.get_market(market_id)?;

        let market_guard = market
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

        let market_guard = market
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
            let market_guard = market
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

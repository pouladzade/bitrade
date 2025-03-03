use anyhow::Result;
use std::collections::HashMap;
use std::sync::{Arc, Mutex};

use crate::models::models::*;
use crate::persistence::persistence::Persistence;
#[derive(Debug, Clone)]
pub struct MockThreadSafePersistence {
    markets: Arc<Mutex<HashMap<String, Market>>>,
    orders: Arc<Mutex<HashMap<String, Order>>>,
    balances: Arc<Mutex<HashMap<(String, String), Balance>>>,
}

impl MockThreadSafePersistence {
    pub fn new() -> Self {
        Self {
            markets: Arc::new(Mutex::new(HashMap::new())),
            orders: Arc::new(Mutex::new(HashMap::new())),
            balances: Arc::new(Mutex::new(HashMap::new())),
        }
    }

    pub fn insert_market(&self, market: Market) {
        self.markets
            .lock()
            .unwrap()
            .insert(market.id.clone(), market);
    }

    pub fn insert_order(&self, order: Order) {
        self.orders.lock().unwrap().insert(order.id.clone(), order);
    }

    pub fn insert_balance(&self, balance: Balance) {
        self.balances
            .lock()
            .unwrap()
            .insert((balance.user_id.clone(), balance.asset.clone()), balance);
    }
}

impl Persistence for MockThreadSafePersistence {
    // fn clone_for_thread(&self) -> Self {
    //     Self {
    //         markets: Arc::clone(&self.markets),
    //         orders: Arc::clone(&self.orders),
    //         balances: Arc::clone(&self.balances),
    //     }
    // }
    fn get_market(&self, market_id: &str) -> Result<Option<Market>> {
        Ok(self.markets.lock().unwrap().get(market_id).cloned())
    }

    fn list_markets(&self) -> Result<Vec<Market>> {
        Ok(self.markets.lock().unwrap().values().cloned().collect())
    }

    fn get_order(&self, order_id: &str) -> Result<Option<Order>> {
        Ok(self.orders.lock().unwrap().get(order_id).cloned())
    }

    fn get_open_orders_for_market(&self, market_id: &str) -> Result<Vec<Order>> {
        let orders = self
            .orders
            .lock()
            .unwrap()
            .values()
            .filter(|o| o.market_id == market_id)
            .cloned()
            .collect();
        Ok(orders)
    }

    fn get_user_orders(&self, user_id: &str, _limit: i64) -> Result<Vec<Order>> {
        let orders = self
            .orders
            .lock()
            .unwrap()
            .values()
            .filter(|o| o.user_id == user_id)
            .cloned()
            .collect();
        Ok(orders)
    }

    fn get_balance(&self, user_id: &str, asset: &str) -> Result<Option<Balance>> {
        Ok(self
            .balances
            .lock()
            .unwrap()
            .get(&(user_id.to_string(), asset.to_string()))
            .cloned())
    }
}

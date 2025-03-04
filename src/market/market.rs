use anyhow::{Context, Result};
use crossbeam::channel;
use database::persistence::persistence::Persistence;
use std::sync::{Arc, RwLock};
use std::thread;

use crate::models::matched_trade::MatchedTrade;
use crate::models::trade_order::TradeOrder;
use crate::order_book::order_book::OrderBook;

type Task<P> = Box<dyn FnOnce(&mut OrderBook<P>) + Send + 'static>;


#[derive(Debug, Clone)]
pub struct Market<P>
where
    P: Persistence + 'static,
{
    task_sender: channel::Sender<Task<P>>,
    persister: Arc<P>,
    order_book: Arc<RwLock<OrderBook<P>>>, // RwLock allows concurrent reads
    started: bool,
    market_id: String,
}

impl<P: Persistence> Market<P> {
    pub fn new(persister: Arc<P>, market_id: String, pool_size: usize) -> Self {
        let (task_sender, task_receiver) = channel::unbounded();
        let order_book = Arc::new(RwLock::new(OrderBook::new(persister.clone()))); // Use RwLock

        for _ in 0..pool_size {
            let task_receiver: channel::Receiver<Task<P>> = task_receiver.clone();
            let order_book = Arc::clone(&order_book);

            thread::spawn(move || {
                while let Ok(task) = task_receiver.recv() {
                    if let Ok(mut order_book) = order_book.write() {
                        task(&mut order_book);
                    } else {
                        panic!("Failed to acquire write lock on order_book");
                    }
                }
            });
        }

        Market {
            task_sender,
            persister,
            order_book,
            started: false,
            market_id,
        }
    }

    pub fn start_market(&mut self) {
        self.started = true;
        println!("Started market {}", self.market_id);
    }

    pub fn stop_market(&mut self) {
        self.started = false;
        print!("Stopped market {}", self.market_id);
    }

    fn submit_task(&self, task: Task<P>) -> Result<()> {
        println!(
            "submit_task market_id: {} started : {}",
            self.market_id, self.started
        );
        if self.started {
            self.task_sender
                .send(task)
                .map_err(|e| anyhow::anyhow!("Failed to send task to worker thread: {}", e))
        } else {
            Err(anyhow::anyhow!("Market is not started"))
        }
    }

    pub fn add_order(&self, order: TradeOrder) -> Result<(Result<Vec<MatchedTrade>,anyhow::Error>, String)> {
        let (sender, receiver) = std::sync::mpsc::channel();
        let order_id = order.id.clone();
        self.submit_task(Box::new(move |order_book: &mut OrderBook<P>|  {
            let trades = order_book.add_order(order);

            let _ = sender.send((trades, order_id));
        }));

        receiver
            .recv()
            .context("Failed to receive order execution result")
    }

    pub fn get_order_by_id(&self, order_id: String) -> Result<Result<Option<TradeOrder>,anyhow::Error>> {
        let order_book = self
            .order_book
            .read()
            .map_err(|e| anyhow::anyhow!("Failed to send task to worker thread: {}", e))?;
        Ok(order_book.get_order_by_id(order_id)) // No need for a task
    }

    pub fn cancel_order(&self, order_id: String) -> Result<Result<bool,anyhow::Error>> {
        let (sender, receiver) = std::sync::mpsc::channel();

        self.submit_task(Box::new(move |order_book: &mut OrderBook<P>| {
            let canceled = order_book.cancel_order(order_id);
            let _ = sender.send(canceled);
        }))?;

        receiver
            .recv()
            .context("Failed to receive order cancellation result")
    }

    pub fn cancel_all_orders(&self) -> Result<Result<bool,anyhow::Error>> {
        let (sender, receiver) = std::sync::mpsc::channel();

        self.submit_task(Box::new(move |order_book: &mut OrderBook<P>| {
            let canceled = order_book.cancel_all_orders();
            let _ = sender.send(canceled);
        }))?;

        receiver
            .recv()
            .context("Failed to receive all orders cancellation result")
    }
}

#[cfg(test)]
mod tests {
    use crate::{
        models::trade_order::{OrderSide, OrderType},
        tests::test_models::{self, create_persistence_mock},
    };

    use super::*;
    const MARKET_ID: &str = "market_id";
    #[test]
    fn test_market_creation() {
        let market = Market::new(create_persistence_mock(),MARKET_ID.to_string(), 4);
        assert_eq!(market.market_id, MARKET_ID);
        assert!(!market.started);
    }

    #[test]
    fn test_start_market() {
        let mut market = Market::new(create_persistence_mock(),MARKET_ID.to_string(), 4);
        market.start_market();
        assert!(market.started);
    }

    #[test]
    fn test_stop_market() {
        let mut market = Market::new(create_persistence_mock(),MARKET_ID.to_string(), 4);
        market.start_market();
        market.stop_market();
        assert!(!market.started);
    }

    #[test]
    fn test_add_order() {
        let mut market = Market::new(create_persistence_mock(),MARKET_ID.to_string(), 4);
        market.start_market();
        let order =
            test_models::create_order(OrderSide::Buy, "100", "10", OrderType::Limit, MARKET_ID);
        let order_id = order.id.clone();
        let result = market.add_order(order);
        assert!(result.is_ok());
        let (res, trade_order_id) = result.unwrap();
        let trades = res.unwrap();
        assert!(trades.is_empty());
        assert_eq!(order_id, trade_order_id);
    }

    #[test]
    fn test_get_order_by_id() {
        let mut market = Market::new(create_persistence_mock(),MARKET_ID.to_string(), 4);
        market.start_market();
        let order =
            test_models::create_order(OrderSide::Buy, "100", "10", OrderType::Limit, MARKET_ID);
        let order_id = order.id.clone();
        market.add_order(order.clone()).unwrap();
        let result = market.get_order_by_id(order_id);
        assert!(result.is_ok());
        let retrieved_order = result.unwrap().unwrap();
        assert!(retrieved_order.is_some());
        assert_eq!(retrieved_order.unwrap(), order);
    }

    #[test]
    fn test_cancel_order() {
        let mut market = Market::new(create_persistence_mock(),MARKET_ID.to_string(), 4);
        market.start_market();
        let order =
            test_models::create_order(OrderSide::Buy, "100", "10", OrderType::Limit, MARKET_ID);
        let order_id = order.id.clone();
        market.add_order(order).unwrap();
        let result = market.cancel_order(order_id);
        assert!(result.is_ok());
        assert!(result.unwrap().unwrap());
    }

    #[test]
    fn test_cancel_all_orders() {
        let mut market = Market::new(create_persistence_mock(),MARKET_ID.to_string(), 4);
        market.start_market();
        let order1 =
            test_models::create_order(OrderSide::Buy, "100", "10", OrderType::Limit, MARKET_ID);
        let order2 =
            test_models::create_order(OrderSide::Buy, "100", "10", OrderType::Limit, MARKET_ID);
        market.add_order(order1).unwrap();
        market.add_order(order2).unwrap();
        let result = market.cancel_all_orders();
        assert!(result.is_ok());
        assert!(result.unwrap().unwrap());
    }
}

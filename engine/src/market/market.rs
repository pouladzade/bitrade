use anyhow::Result;
use crossbeam::channel;
use database::persistence::persistence::Persistence;
use std::sync::atomic::{AtomicBool, Ordering};
use std::sync::{Arc, Mutex};
use std::thread;

use crate::models::matched_trade::MatchedTrade;
use crate::models::trade_order::TradeOrder;
use crate::order_book::order_book::OrderBook;

/// Custom error type for market-related failures
#[derive(Debug, thiserror::Error)]
pub enum MarketError {
    #[error("Failed to send task to order book thread")]
    TaskSendError,

    #[error("Market is not started")]
    MarketNotStarted,

    #[error("Failed to receive response from order book")]
    ResponseReceiveError,

    #[error("Market is already started")]
    MarketAlreadyStarted,
}

type Task<P> = Box<dyn FnOnce(&mut OrderBook<P>) + Send + 'static>;

#[derive(Debug)]
pub struct Market<P>
where
    P: Persistence + 'static,
{
    task_sender: channel::Sender<Task<P>>,
    persister: Arc<P>,
    market_id: String,
    base_asset: String,
    quote_asset: String,
    started: Arc<AtomicBool>, // Track market status
}

impl<P: Persistence> Market<P> {
    pub fn new(
        persister: Arc<P>,
        market_id: String,
        base_asset: String,
        quote_asset: String,
    ) -> Result<Self> {
        let (task_sender, task_receiver): (channel::Sender<Task<P>>, channel::Receiver<Task<P>>) =
            channel::unbounded();

        let started = Arc::new(AtomicBool::new(false));

        let persister_clone = Arc::clone(&persister);
        let started_clone = Arc::clone(&started);
        let base_asset_clone = base_asset.clone();
        let market_id_clone = market_id.clone();
        let quote_asset_clone = quote_asset.clone();
        thread::spawn(move || {
            let mut order_book = OrderBook::new(
                persister_clone,
                base_asset_clone,
                market_id_clone,
                quote_asset_clone,
            );
            while let Ok(task) = task_receiver.recv() {
                match started_clone.load(Ordering::SeqCst) {
                    true => task(&mut order_book),
                    false => break, // Stop processing if market is stopped
                }
            }
        });

        Ok(Self {
            task_sender,
            persister,
            market_id,
            started,
            base_asset,
            quote_asset,
        })
    }

    pub fn get_market_id(&self) -> String {
        self.market_id.clone()
    }

    pub fn start_market(&self) -> Result<()> {
        if self.started.load(Ordering::SeqCst) {
            return Err(MarketError::MarketAlreadyStarted.into());
        }

        self.started.store(true, Ordering::SeqCst);
        println!("Market {} started", self.market_id);
        Ok(())
    }

    pub fn stop_market(&self) -> Result<()> {
        if !self.started.load(Ordering::SeqCst) {
            return Err(MarketError::MarketNotStarted.into());
        }
        self.started.store(false, Ordering::SeqCst);
        println!("Market {} stopped", self.market_id);
        Ok(())
    }

    fn submit_task(&self, task: Task<P>) -> Result<()> {
        if self.started.load(Ordering::SeqCst) {
            self.task_sender.send(task).map_err(|_| {
                anyhow::anyhow!("Failed to send task").context(MarketError::TaskSendError)
            })
        } else {
            Err(
                anyhow::anyhow!("Cannot submit task while market is stopped")
                    .context(MarketError::MarketNotStarted),
            )
        }
    }

    pub fn add_order(&self, order: TradeOrder) -> Result<Vec<MatchedTrade>> {
        let (sender, receiver) = std::sync::mpsc::channel();

        self.submit_task(Box::new(move |order_book: &mut OrderBook<P>| {
            let trades = order_book.add_order(order);
            let _ = sender.send(trades);
        }))?;

        receiver.recv()?
    }

    pub fn get_order_by_id(&self, order_id: String) -> Result<TradeOrder> {
        let (sender, receiver) = std::sync::mpsc::channel();

        let _ = self.submit_task(Box::new(move |order_book: &mut OrderBook<P>| {
            let result = order_book.get_order_by_id(order_id);
            let _ = sender.send(result);
        }));

        receiver.recv()?
    }

    pub fn cancel_order(&self, order_id: String) -> Result<bool> {
        let (sender, receiver) = std::sync::mpsc::channel();

        self.submit_task(Box::new(move |order_book: &mut OrderBook<P>| {
            let canceled = order_book.cancel_order(order_id);
            let _ = sender.send(canceled);
        }))?;

        receiver.recv()?
    }

    pub fn cancel_all_orders(&self) -> Result<bool> {
        let (sender, receiver) = std::sync::mpsc::channel();

        self.submit_task(Box::new(move |order_book: &mut OrderBook<P>| {
            let canceled = order_book.cancel_all_orders();
            let _ = sender.send(canceled);
        }))?;

        receiver.recv()?
    }
}


use crate::models::order::Order;
use crossbeam_channel::{unbounded, Receiver, Sender};

use super::shared_order_book::SharedOrderBook;

pub struct Market {
    order_book: SharedOrderBook,
    order_tx: Sender<Order>,
    order_rx: Receiver<Order>,
}

impl Market {
    pub fn new() -> Self {
        let (order_tx, order_rx) = unbounded();
        Market {
            order_book: SharedOrderBook::new(),
            order_tx,
            order_rx,
        }
    }

    pub async fn run(&self) {
        let order_book = self.order_book.clone();
        let order_rx = self.order_rx.clone();

        tokio::spawn(async move {
            while let Ok(order) = order_rx.recv() {
                let _trades = order_book.add_order(order).await;
                // Handle trades (e.g., persist to database, notify users)
            }
        });
    }

    pub fn submit_order(&self, order: Order) {
        self.order_tx.send(order).unwrap();
    }
}

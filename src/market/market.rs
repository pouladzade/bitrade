use crossbeam::channel;
use std::sync::{Arc, RwLock};
use std::thread;

use crate::models::order::Order;
use crate::models::trade::Trade;
use crate::order_book::order_book::OrderBook;
use crate::order_book::order_book::OrderBookTrait;

type Task = Box<dyn FnOnce(&mut OrderBook) + Send + 'static>;

#[derive(Debug, Clone)]
pub struct Market {
    task_sender: channel::Sender<Task>,
    order_book: Arc<RwLock<OrderBook>>, // RwLock allows concurrent reads
}

impl Market {
    pub fn new(pool_size: usize) -> Self {
        let (task_sender, task_receiver) = channel::unbounded();
        let order_book = Arc::new(RwLock::new(OrderBook::new())); // Use RwLock

        for _ in 0..pool_size {
            let task_receiver: channel::Receiver<Task> = task_receiver.clone();
            let order_book = Arc::clone(&order_book);

            thread::spawn(move || {
                while let Ok(task) = task_receiver.recv() {
                    let mut order_book = order_book.write().unwrap(); // Write lock
                    task(&mut order_book);
                }
            });
        }

        Market {
            task_sender,
            order_book,
        }
    }

    fn submit_task(&self, task: Task) {
        self.task_sender.send(task).unwrap();
    }

    pub fn add_order(&self, order: Order) -> Vec<Trade> {
        let (sender, receiver) = std::sync::mpsc::channel();
        self.submit_task(Box::new(move |order_book: &mut OrderBook| {
            let trades = order_book.add_order(order);
            sender.send(trades).unwrap();
        }));
        println!("Market Added order");
        receiver.recv().unwrap()
    }

    pub fn get_order_by_id(&self, order_id: u64) -> Option<Order> {
        let order_book = self.order_book.read().unwrap(); // Read lock
        order_book.get_order_by_id(order_id) // No need for a task
    }

    pub fn cancel_order(&self, order_id: u64) -> bool {
        let (sender, receiver) = std::sync::mpsc::channel();
        self.submit_task(Box::new(move |order_book: &mut OrderBook| {
            let canceled = order_book.cancel_order(order_id);
            sender.send(canceled).unwrap();
        }));
        receiver.recv().unwrap()
    }

    pub fn cancel_all_orders(&self) {
        let (sender, receiver) = std::sync::mpsc::channel();
        self.submit_task(Box::new(move |order_book: &mut OrderBook| {
            order_book.cancel_all_orders();
            sender.send(()).unwrap();
        }));
        receiver.recv().unwrap();
    }
}

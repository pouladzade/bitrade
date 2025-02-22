use super::order_book::OrderBook;
use crate::models::order::Order;
use crate::models::trade::Trade;
use super::order_book::OrderBookTrait;
use std::sync::{Arc, Mutex};

#[derive(Debug, Clone)]
pub struct SharedOrderBook {
    inner: Arc<Mutex<OrderBook>>,
}

impl SharedOrderBook {
    pub fn new() -> Self {
        Self {
            inner: Arc::new(Mutex::new(OrderBook::new())),
        }
    }

    pub async fn add_order(&self, order: Order) -> Vec<Trade> {
        let mut order_book = self.inner.lock().unwrap();
        order_book.add_order(order)
    }

    pub async fn cancel_order(&self, order_id: u64) -> bool {
        let mut order_book = self.inner.lock().unwrap();
        order_book.cancel_order(order_id)
    }

    pub async fn get_order_by_id(&self, order_id: u64) -> Option<Order> {
        let order_book = self.inner.lock().unwrap();
        order_book.get_order_by_id(order_id)
    }

    pub async fn cancel_all_orders(&self) {
        let mut order_book = self.inner.lock().unwrap();
        order_book.cancel_all_orders();
    }
}

#[cfg(test)]
mod tests {
    use rust_decimal::Decimal;
    use std::str::FromStr;
    use tokio::task;

    use crate::models::order::{OrderSide, OrderType};

    use super::*;
    fn create_order(
        id: u64,
        side: OrderSide,
        price: &str,
        amount: &str,
        create_time: f64,
        order_type: OrderType,
    ) -> Order {
        Order {
            id,
            base_asset: "BTC".into(),
            quote_asset: "USD".into(),
            market: "BTC-USD".into(),
            order_type,
            side,
            user_id: 1,
            post_only: false,
            price: Decimal::from_str(price).unwrap(),
            amount: Decimal::from_str(amount).unwrap(),
            maker_fee: Decimal::ZERO,
            taker_fee: Decimal::ZERO,
            create_time,
            remain: Decimal::from_str(amount).unwrap(),
            frozen: Decimal::ZERO,
            filled_base: Decimal::ZERO,
            filled_quote: Decimal::ZERO,
            filled_fee: Decimal::ZERO,
            update_time: create_time,
            partially_filled: true,
        }
    }
    #[tokio::test]
    async fn test_add_order() {
        let shared_order_book = SharedOrderBook::new();
        let order = create_order(1, OrderSide::Buy, "50000", "1", 1.0, OrderType::Limit);
        let trades = shared_order_book.add_order(order).await;
        assert!(trades.is_empty());
    }

    #[tokio::test]
    async fn test_cancel_order() {
        let shared_order_book = SharedOrderBook::new();
        let order = create_order(1, OrderSide::Buy, "100", "10", 1.0, OrderType::Limit);
        shared_order_book.add_order(order.clone()).await;
        let canceled = shared_order_book.cancel_order(1).await;
        assert_eq!(canceled, true);
    }

    #[tokio::test]
    async fn test_add_multiple_orders() {
        let shared_order_book = SharedOrderBook::new();
        let order1 = create_order(1, OrderSide::Buy, "100", "10", 1.0, OrderType::Limit);
        let order2 = create_order(2, OrderSide::Sell, "100", "10", 1.0, OrderType::Limit);
        let trades1 = shared_order_book.add_order(order1).await;
        let trades2 = shared_order_book.add_order(order2).await;
        assert!(trades1.is_empty());
        assert_eq!(trades2.len(), 1);
        assert_eq!(trades2[0].bid_order_id, 1);
        assert_eq!(trades2[0].ask_order_id, 2);
    }

    #[tokio::test]
    async fn test_concurrent_access() {
        let shared_order_book = Arc::new(SharedOrderBook::new());
        let order1 = create_order(1, OrderSide::Buy, "100", "10", 1.0, OrderType::Limit);
        let order2 = create_order(2, OrderSide::Sell, "100", "10", 1.0, OrderType::Limit);

        let shared_order_book_clone1 = Arc::clone(&shared_order_book);
        let shared_order_book_clone2 = Arc::clone(&shared_order_book);

        let handle1 = task::spawn(async move {
            shared_order_book_clone1.add_order(order1).await;
        });

        let handle2 = task::spawn(async move {
            shared_order_book_clone2.add_order(order2).await;
        });

        handle1.await.unwrap();
        handle2.await.unwrap();

        let order_book = shared_order_book.inner.lock().unwrap();
        assert!(order_book.get_order_by_id(1).is_none());
        assert!(order_book.get_order_by_id(2).is_none());
    }
}

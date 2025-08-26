use crate::market::market_manager::MarketManager;
use crate::models::trade_order::{OrderSide, OrderType, TradeOrder};
use crate::tests::test_models::create_order;
use database::mock::mock_thread_safe_persistence::MockThreadSafePersistence;
use std::sync::Arc;

#[test]
fn test_market_manager_creation() {
    let mock_persister = Arc::new(MockThreadSafePersistence::new());
    let market_manager = MarketManager::new(mock_persister);

    // Verify market manager is created successfully
    assert!(market_manager.markets.lock().unwrap().is_empty());
}

#[test]
fn test_create_market() {
    let mock_persister = Arc::new(MockThreadSafePersistence::new());
    let market_manager = MarketManager::new(mock_persister);

    let result = market_manager.create_market(
        "BTC-USD".to_string(),
        "BTC".to_string(),
        "USD".to_string(),
        "0.001".to_string(),
        "0.002".to_string(),
    );

    assert!(result.is_ok());

    // Verify market was created
    let markets = market_manager.markets.lock().unwrap();
    assert!(markets.contains_key("BTC-USD"));
}

#[test]
fn test_add_order() {
    let mock_persister = Arc::new(MockThreadSafePersistence::new());
    let market_manager = MarketManager::new(mock_persister);

    // Create a market first
    market_manager
        .create_market(
            "BTC-USD".to_string(),
            "BTC".to_string(),
            "USD".to_string(),
            "0.001".to_string(),
            "0.002".to_string(),
        )
        .unwrap();

    // Create a buy order
    let buy_order = create_order(
        OrderSide::Buy,
        "50000",
        "1",
        "50000",
        OrderType::Limit,
        "BTC-USD",
    );

    let result = market_manager.add_order(buy_order);
    assert!(result.is_ok());

    let (trades, market_id) = result.unwrap();
    assert_eq!(trades.len(), 0); // No trades yet
    assert_eq!(market_id, "BTC-USD");
}

#[test]
fn test_cancel_order() {
    let mock_persister = Arc::new(MockThreadSafePersistence::new());
    let market_manager = MarketManager::new(mock_persister);

    // Create a market first
    market_manager
        .create_market(
            "BTC-USD".to_string(),
            "BTC".to_string(),
            "USD".to_string(),
            "0.001".to_string(),
            "0.002".to_string(),
        )
        .unwrap();

    // Create and add a buy order
    let buy_order = create_order(
        OrderSide::Buy,
        "50000",
        "1",
        "50000",
        OrderType::Limit,
        "BTC-USD",
    );

    let (_, _) = market_manager.add_order(buy_order.clone()).unwrap();

    // Cancel the order
    let result = market_manager.cancel_order("BTC-USD", buy_order.id);
    assert!(result.is_ok());
    assert!(result.unwrap());
}

#[test]
fn test_cancel_all_orders() {
    let mock_persister = Arc::new(MockThreadSafePersistence::new());
    let market_manager = MarketManager::new(mock_persister);

    // Create a market first
    market_manager
        .create_market(
            "BTC-USD".to_string(),
            "BTC".to_string(),
            "USD".to_string(),
            "0.001".to_string(),
            "0.002".to_string(),
        )
        .unwrap();

    // Create and add multiple orders
    let buy_order = create_order(
        OrderSide::Buy,
        "50000",
        "1",
        "50000",
        OrderType::Limit,
        "BTC-USD",
    );

    let sell_order = create_order(
        OrderSide::Sell,
        "51000",
        "1",
        "51000",
        OrderType::Limit,
        "BTC-USD",
    );

    market_manager.add_order(buy_order).unwrap();
    market_manager.add_order(sell_order).unwrap();

    // Cancel all orders
    let result = market_manager.cancel_all_orders("BTC-USD");
    assert!(result.is_ok());
    assert!(result.unwrap());
}

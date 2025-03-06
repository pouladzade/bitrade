use crate::models::matched_trade::MatchedTrade;
use crate::models::trade_order::{determine_order_status, OrderSide, OrderType, TradeOrder};
use crate::utils::is_zero;
use crate::wallet::wallet::Wallet;
use anyhow::{Ok, Result};
use bigdecimal::BigDecimal;
use colored::*;
use database::models::models::{NewOrder, NewTrade};
use database::persistence::persistence::Persistence;
use std::collections::BinaryHeap;
use std::sync::Arc;

#[derive(Debug, Clone)]
pub struct OrderBook<P>
where
    P: Persistence + 'static,
{
    bids: BinaryHeap<TradeOrder>, // Max-heap for bids (buy orders)
    asks: BinaryHeap<TradeOrder>, // Min-heap for asks (sell orders)
    persister: Arc<P>,
    market_price: BigDecimal,
    base_asset: String,
    quote_asset: String,
    market_id: String,
}

impl<P: Persistence> OrderBook<P> {
    /// Add a new order asynchronously
    pub fn new(
        persister: Arc<P>,
        base_asset: String,
        market_id: String,
        quote_asset: String,
    ) -> Self {
        let mut order_book = OrderBook {
            bids: BinaryHeap::new(),
            asks: BinaryHeap::new(),
            base_asset,
            quote_asset,
            market_id,
            persister,
            //TODO: Get the market price from the database OR from the market
            market_price: BigDecimal::try_from(10000).unwrap(),
        };

        order_book.load_orders_from_db().unwrap();
        order_book
    }

    pub fn load_orders_from_db(&mut self) -> Result<()> {
        let orders = self.persister.get_active_orders(&self.market_id)?;
        let orders_len = orders.len();
        if orders_len > 0 {
            self.market_price = orders[0].price.clone();
        } else {
            //TODO: Get the market price from the database OR from the market
            self.market_price = BigDecimal::try_from(10000).unwrap();
        }
        for order in orders {
            let trade_order: TradeOrder = order.try_into()?;
            self.match_order(trade_order)?;
        }
        println!("Loaded {} orders from database", orders_len);
        Ok(())
    }
    pub fn add_order(&mut self, order: TradeOrder) -> anyhow::Result<Vec<MatchedTrade>> {
        Self::print_order(&order);
        self.persist_create_order(&order)?;
        self.match_order(order)
    }
    fn match_order(&mut self, mut order: TradeOrder) -> anyhow::Result<Vec<MatchedTrade>> {
        let mut trades = Vec::new();

        Self::print_order(&order);

        match order.side {
            OrderSide::Buy => {
                // Try to match the buy order with existing sell orders (asks)
                while let Some(mut ask) = self.asks.pop() {
                    // Stop if the ask price is higher than the buy order price for Limit orders
                    if order.order_type == OrderType::Limit
                        && ask.order_type == OrderType::Limit
                        && ask.price > order.price
                    {
                        // No more matching asks
                        self.asks.push(ask); // Push it back to the heap
                        break;
                    }
                    //TODO: check if ask and bid orders are from the same user
                    // if order.user_id == ask.user_id {
                    //     println!("User id is the same");
                    //     self.cancel_order(ask.id)?;
                    //     continue;
                    // }

                    // Calculate the trade amount
                    let trade_amount = order.remain.clone().min(ask.remain.clone());

                    // Execute the trade
                    let trade = self.execute_trade(&mut order, &mut ask, trade_amount)?;
                    trades.push(trade);

                    // Remove the ask order if fully filled
                    if ask.remain != BigDecimal::from(0) {
                        self.asks.push(ask); // Push the modified ask back into the heap
                    }

                    // Stop if the buy order is fully filled
                    if is_zero(&order.remain) {
                        break;
                    }
                }

                // Add the remaining buy order to the order book
                if !is_zero(&order.remain) {
                    self.bids.push(order.clone());
                }
            }
            OrderSide::Sell => {
                // Try to match the sell order with existing buy orders (bids)
                while let Some(mut bid) = self.bids.pop() {
                    // Stop if the bid price is lower than the sell order price for Limit orders
                    if order.order_type == OrderType::Limit
                        && bid.order_type == OrderType::Limit
                        && bid.price < order.price
                    {
                        print!("{} < {}", bid.price, order.price);
                        // No more matching bids
                        self.bids.push(bid); // Push it back to the heap
                        break;
                    }

                    //TODO: check if ask and bid orders are from the same user
                    // if order.user_id == bid.user_id {
                    //     println!("User id is the same");
                    //     self.cancel_order(bid.id)?;
                    //     continue;
                    // }

                    // Calculate the trade amount
                    let trade_amount = order.remain.clone().min(bid.remain.clone());
                    // Execute the trade
                    let trade = self.execute_trade(&mut order, &mut bid, trade_amount)?;
                    trades.push(trade);

                    if !is_zero(&bid.remain) {
                        self.bids.push(bid); // Push the modified bid back into the heap
                    }

                    // Stop if the sell order is fully filled
                    if is_zero(&order.remain) {
                        break;
                    }
                }

                // Add the remaining sell order to the order book
                if !is_zero(&order.remain) {
                    self.asks.push(order.clone());
                }
            }
        }
        self.print_order_book();
        Ok(trades)
    }

    /// Cancel an order by its ID.
    pub fn cancel_order(&mut self, order_id: String) -> anyhow::Result<bool> {
        self.persister.cancel_order(&order_id)?;
        let bids_initial_len = self.bids.len();
        self.bids.retain(|o| o.id != order_id);
        if self.bids.len() != bids_initial_len {
            return Ok(true);
        }
        let asks_initial_len = self.asks.len();
        self.asks.retain(|o| o.id != order_id);
        if self.asks.len() != asks_initial_len {
            return Ok(true);
        }
        return Ok(false);
    }
    pub fn get_order_by_id(&self, order_id: String) -> anyhow::Result<TradeOrder> {
        if let Some(order) = self.bids.iter().find(|o| o.id == order_id) {
            return Ok(order.clone());
        } else if let Some(order) = self.asks.iter().find(|o| o.id == order_id) {
            return Ok(order.clone());
        }
        Err(anyhow::anyhow!("can not find the order!"))
    }

    pub fn cancel_all_orders(&mut self) -> anyhow::Result<bool> {
        self.persister.cancel_all_orders(&self.market_id)?;
        self.bids.clear();
        self.asks.clear();
        Ok(true)
    }

    fn execute_trade(
        &mut self,
        taker: &mut TradeOrder,
        maker: &mut TradeOrder,
        amount: BigDecimal,
    ) -> anyhow::Result<MatchedTrade> {
        let price = self.calculate_trade_price(taker, maker)?;
        let buyer_fee: BigDecimal;
        let seller_fee: BigDecimal;
        let (is_buyer_taker, buyer, seller) = match taker.side == OrderSide::Buy {
            true => {
                buyer_fee = taker.taker_fee.clone();
                seller_fee = maker.maker_fee.clone();
                (true, taker.clone(), maker.clone())
            }
            false => {
                seller_fee = taker.taker_fee.clone();
                buyer_fee = maker.maker_fee.clone();
                (false, maker.clone(), taker.clone())
            }
        };

        let quote_amount = amount.clone() * price.clone();

        // // Update remaining amounts after deducting fees
        taker.remain -= amount.clone();
        taker.filled_base += amount.clone();
        taker.filled_quote += quote_amount.clone();
        taker.filled_fee += taker.taker_fee.clone();

        maker.remain -= amount.clone();
        maker.filled_base += amount.clone();
        maker.filled_quote += quote_amount.clone();
        maker.filled_fee += maker.maker_fee.clone();

        let trade_data = self.persister.execute_trade(
            is_buyer_taker,
            self.market_id.clone(),
            self.base_asset.clone(),
            self.quote_asset.clone(),
            buyer.user_id,
            seller.user_id,
            buyer.id,
            seller.id,
            price.clone(),
            amount,
            quote_amount,
            buyer_fee,
            seller_fee,
        )?;
        self.market_price = price;
        // Construct the trade object
        let trade = MatchedTrade {
            id: trade_data.id,
            timestamp: trade_data.timestamp,
            market_id: trade_data.market_id,
            price: trade_data.price,
            amount: trade_data.amount,
            quote_amount: trade_data.quote_amount,
            maker_user_id: trade_data.maker_user_id,
            maker_order_id: trade_data.maker_order_id,
            maker_fee: trade_data.maker_fee,
            taker_user_id: trade_data.taker_user_id,
            taker_order_id: trade_data.taker_order_id,
            taker_fee: trade_data.taker_fee,
        };

        // Log trade execution
        Self::print_trade(&trade);
        // everything is done inside execute trade function so no need to call these functions her
        Ok(trade)
    }

    fn calculate_trade_price(
        &self,
        taker: &TradeOrder,
        maker: &TradeOrder,
    ) -> anyhow::Result<BigDecimal> {
        match (taker.order_type, maker.order_type) {
            // Market orders trade at the market price
            (OrderType::Market, OrderType::Market) => Ok(self.market_price.clone()),

            // Market order takes the price of the existing Limit order
            (OrderType::Market, OrderType::Limit) => Ok(maker.price.clone()),
            (OrderType::Limit, OrderType::Market) => Ok(taker.price.clone()),

            // Limit orders always execute at the maker's price
            (OrderType::Limit, OrderType::Limit) => Ok(maker.price.clone()),
        }
    }

    fn persist_create_order(&self, order: &TradeOrder) -> anyhow::Result<()> {
        let new_order: NewOrder = order.clone().into(); // Convert TradeOrder to NewOrder

        self.persister.create_order(new_order)?;

        Ok(())
    }

    pub fn print_bids(&self) {
        let bids_sorted: Vec<TradeOrder> = self.bids.clone().into_sorted_vec();
        let bids_reversed: Vec<TradeOrder> = bids_sorted.into_iter().rev().collect();
        for bid in bids_reversed {
            let price = match bid.order_type {
                OrderType::Market => "Market".to_string(),
                _ => bid.price.to_string(),
            };
            println!(
                "{} {} , {} {} , {} {} , {} {} , {} {} , {} {}",
                "id:".green(),
                bid.id,
                "price:".green(),
                price,
                "amount:".green(),
                bid.amount,
                "remain:".green(),
                bid.remain,
                "Type:".blue(),
                String::from(bid.order_type),
                "user:".blue(),
                bid.user_id
            );
        }
    }

    pub fn print_asks(&self) {
        let asks_sorted: Vec<TradeOrder> = self.asks.clone().into_sorted_vec();
        let asks_reversed: Vec<TradeOrder> = asks_sorted.into_iter().rev().collect();
        for ask in asks_reversed {
            let price = match ask.order_type {
                OrderType::Market => "Market".to_string(),
                _ => ask.price.to_string(),
            };

            println!(
                "{} {} , {} {} , {} {} , {} {} , {} {} , {} {}",
                "id:".red(),
                ask.id,
                "price:".red(),
                price,
                "amount:".red(),
                ask.amount,
                "remain:".red(),
                ask.remain,
                "Type:".blue(),
                String::from(ask.order_type),
                "user:".blue(),
                ask.user_id
            );
        }
    }

    fn print_order_book(&self) {
        println!("\n{}", "Order Book:".bold().white());
        println!("{}", "Bids (Buy Orders):".green().bold());
        self.print_bids();
        println!("{}", "Asks (Sell Orders):".red().bold());
        self.print_asks();
    }

    fn print_order(order: &TradeOrder) {
        println!(
            "\nNew Order Arrived {} {} , {} {} , {} {}, {} {}",
            "Order id:".blue(),
            order.id,
            "price:".blue(),
            order.price,
            "amount:".blue(),
            order.amount,
            "Type:".blue(),
            String::from(order.order_type)
        );
    }

    fn print_trade(trade: &MatchedTrade) {
        println!(
            "\nNew Trade Matched {} {} , {} {} , {} {} , {} {}",
            "Trade id:".cyan(),
            trade.id,
            "price:".cyan(),
            trade.price,
            "amount:".cyan(),
            trade.amount,
            "quote_amount:".cyan(),
            trade.quote_amount
        );
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::{
        models::trade_order::{OrderSide, OrderType, TradeOrder},
        tests::test_models::create_persistence_mock,
    };
    use anyhow::Context;
    use bigdecimal::BigDecimal;
    use database::mock::mock_thread_safe_persistence::MockThreadSafePersistence;
    use env_logger;
    use std::str::FromStr;

    fn create_order(
        id: &str,
        side: OrderSide,
        price: &str,
        amount: &str,
        create_time: i64,
        order_type: OrderType,
    ) -> TradeOrder {
        TradeOrder {
            id: id.to_string(),
            market_id: "BTC-USD".into(),
            order_type,
            side,
            user_id: "1".to_string(),
            price: BigDecimal::from_str(price).unwrap(),
            amount: BigDecimal::from_str(amount).unwrap(),
            maker_fee: BigDecimal::from(0),
            taker_fee: BigDecimal::from(0),
            create_time,
            remain: BigDecimal::from_str(amount).unwrap(),

            filled_base: BigDecimal::from(0),
            filled_quote: BigDecimal::from(0),
            filled_fee: BigDecimal::from(0),
            update_time: create_time,
        }
    }

    // #[test]
    // fn test_order_book_matching() {
    //     env_logger::init();
    //     let persister = MockThreadSafePersistence::new();
    //     let mut order_book = OrderBook::new(create_persistence_mock());

    //     // Add a bid (buy order)
    //     let bid = create_order("1", OrderSide::Buy, "50000", "1", 1, OrderType::Limit);
    //     let trades = order_book
    //         .add_order(bid)
    //         .context("can't add order")
    //         .unwrap();
    //     assert_eq!(trades.len(), 0); // No trades yet

    //     // Add an ask (sell order) that matches the bid
    //     let ask = create_order("2", OrderSide::Sell, "50000", "1", 2, OrderType::Limit);
    //     let trades = order_book
    //         .add_order(ask)
    //         .context("can't add order")
    //         .unwrap();
    //     assert_eq!(trades.len(), 1); // One trade should occur
    //     println!("{:?}", trades);
    //     // Verify the trade details
    //     let trade = &trades[0];
    //     assert_eq!(trade.price, BigDecimal::from_str("50000").unwrap());
    //     assert_eq!(trade.amount, BigDecimal::from_str("1").unwrap());
    //     assert_eq!(trade.taker_order_id, "2");
    //     assert_eq!(trade.maker_order_id, "1");

    //     // Verify the order book is empty after the match
    //     assert!(order_book.bids.is_empty());
    //     assert!(order_book.asks.is_empty());
    // }

    // #[test]
    // fn test_partial_match() {
    //     let mut order_book = OrderBook::new(create_persistence_mock());

    //     // Add a bid (buy order)
    //     let bid = create_order("1", OrderSide::Buy, "50000", "2", 1, OrderType::Limit);
    //     let trades = order_book.add_order(bid).unwrap();
    //     assert_eq!(trades.len(), 0); // No trades yet

    //     // Add an ask (sell order) that partially matches the bid
    //     let ask = create_order("2", OrderSide::Sell, "50000", "1", 2, OrderType::Limit);
    //     let trades = order_book.add_order(ask).unwrap();
    //     assert_eq!(trades.len(), 1); // One trade should occur
    //     println!("{:?}", trades);
    //     // Verify the trade details
    //     let trade = &trades[0];
    //     assert_eq!(trade.price, BigDecimal::from_str("50000").unwrap());
    //     assert_eq!(trade.amount, BigDecimal::from_str("1").unwrap());

    //     // Verify the remaining bid in the order book
    //     // assert_eq!(order_book.asks.len(), 1);
    //     assert_eq!(order_book.bids.len(), 1);
    //     let remaining_bid = order_book.bids.peek().unwrap();
    //     println!("{:?}", remaining_bid);
    //     assert_eq!(remaining_bid.id, "1");
    //     assert_eq!(remaining_bid.remain, BigDecimal::from_str("1").unwrap());

    //     // Verify the ask is fully filled and removed
    //     assert!(order_book.asks.is_empty());
    // }

    // #[test]
    // fn test_cancel_order() {
    //     let mut order_book = OrderBook::new(create_persistence_mock());

    //     // Add a bid (buy order)
    //     let bid = create_order("1", OrderSide::Buy, "50000", "1", 1, OrderType::Limit);
    //     order_book.add_order(bid);

    //     // Cancel the bid
    //     let canceled = order_book.cancel_order("1".to_string()).unwrap();
    //     assert_eq!(canceled, true);

    //     // Verify the bid is removed from the order book
    //     assert!(order_book.bids.is_empty());
    // }
}

use crate::models::matched_trade::MatchedTrade;
use crate::models::trade_order::{OrderType, TradeOrder};
use crate::order_book::OrderBook;
use colored::*;
use database::provider::DatabaseProvider;
impl<P: DatabaseProvider> OrderBook<P> {
    pub fn print_bids(&self) {
        let bids_sorted: Vec<TradeOrder> = self.bids.clone().into_sorted_vec();
        let bids_reversed: Vec<TradeOrder> = bids_sorted.into_iter().rev().collect();
        for bid in bids_reversed {
            let price = match bid.order_type {
                OrderType::Market => "Market".to_string(),
                _ => bid.price.to_string(),
            };
            println!(
                "{} {} , {} {} , {} {} , {} {} , {} {}",
                "id:".green(),
                bid.id,
                "price:".green(),
                price,
                "amount:".green(),
                bid.base_amount,
                "remain:".green(),
                bid.remained_base,
                String::from(bid.order_type).blue(),
                bid.user_id.blue()
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
                "{} {} , {} {} , {} {} , {} {} , {} {}",
                "id:".red(),
                ask.id,
                "price:".red(),
                price,
                "amount:".red(),
                ask.base_amount,
                "remain:".red(),
                ask.remained_base,
                String::from(ask.order_type).blue(),
                ask.user_id.blue()
            );
        }
    }

    pub fn print_order_book(&self) {
        println!("\n{}", "Order Book:".bold().white());
        println!("{}", "Bids (Buy Orders):".green().bold());
        self.print_bids();
        println!("{}", "Asks (Sell Orders):".red().bold());
        self.print_asks();
        println!("{}", "Depth:".bold().white());
        self.print_depth();
    }

    pub fn print_order(order: &TradeOrder) {
        println!(
            "\nNew Order Arrived {} {} , {} {} , {} {}, {} {}",
            "Order id:".blue(),
            order.id,
            "price:".blue(),
            order.price,
            "amount:".blue(),
            order.base_amount,
            "Type:".blue(),
            String::from(order.order_type)
        );
    }

    pub fn print_trade(trade: &MatchedTrade) {
        println!(
            "\nNew Trade Matched {} {} , {} {} , {} {} , {} {}",
            "Trade id:".cyan(),
            trade.id,
            "price:".cyan(),
            trade.price,
            "base_amount:".cyan(),
            trade.base_amount,
            "quote_amount:".cyan(),
            trade.quote_amount
        );
    }

    pub fn print_asks_depth(&self) {
        self.ask_depth.iter().for_each(|(price, amount)| {
            println!("{} {}", price, amount);
        });
    }

    pub fn print_bids_depth(&self) {
        self.bid_depth.iter().for_each(|(price, amount)| {
            println!("{} {}", price, amount);
        });
    }

    pub fn print_depth(&self) {
        println!("Bids Depth: ");
        self.print_bids_depth();
        println!("Asks Depth: ");
        self.print_asks_depth();
    }
}

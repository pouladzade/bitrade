use super::OrderBook;
use crate::models::matched_trade::MatchedTrade;
use crate::models::trade_order::{OrderSide, OrderType, TradeOrder};
use bigdecimal::BigDecimal;
use common::utils::is_zero;
use database::provider::DatabaseProvider;

impl<P: DatabaseProvider> OrderBook<P> {
    pub fn match_limit_order(
        &mut self,
        mut order: TradeOrder,
    ) -> anyhow::Result<Vec<MatchedTrade>> {
        let mut trades = Vec::new();

        Self::print_order(&order);
        // Add to depth maps before matching
        self.handle_market_depth(&order);
        match order.side {
            OrderSide::Buy => {
                // Try to match the buy order with existing sell orders (asks)
                while let Some(mut ask) = self.asks.pop() {
                    // Stop if the ask price is higher than the buy order price for Limit orders
                    if ask.price > order.price {
                        // No more matching asks
                        self.asks.push(ask); // Push it back to the heap
                        break;
                    }

                    // Calculate the trade amount
                    let trade_price = self.calculate_trade_price(&order, &ask, true)?;
                    let trade_amount = self.calculate_trade_amount(&order, &ask, &trade_price)?;

                    // Execute the trade
                    let trade =
                        self.execute_trade(&mut order, &mut ask, trade_amount, trade_price, true)?;
                    trades.push(trade);

                    // Remove the ask order if fully filled
                    if !is_zero(&ask.remained_base) {
                        self.asks.push(ask); // Push the modified ask back into the heap
                    }

                    // Stop if the buy order is fully filled
                    if is_zero(&order.remained_base) {
                        break;
                    }
                }

                // Add the remaining buy order to the order book and update depth
                if !is_zero(&order.remained_base) {
                    self.bids.push(order.clone());
                }
            }
            OrderSide::Sell => {
                // Try to match the sell order with existing buy orders (bids)
                while let Some(mut bid) = self.bids.pop() {
                    // Stop if the bid price is lower than the sell order price for Limit orders
                    if bid.price < order.price {
                        // No more matching bids
                        self.bids.push(bid); // Push it back to the heap
                        break;
                    }

                    let trade_price = self.calculate_trade_price(&bid, &order, false)?;
                    // Calculate the trade amount
                    let trade_amount = self.calculate_trade_amount(&bid, &order, &trade_price)?;

                    // Execute the trade
                    let trade = self.execute_trade(
                        &mut bid,
                        &mut order,
                        trade_amount.clone(),
                        trade_price,
                        false,
                    )?;
                    trades.push(trade);

                    if !is_zero(&bid.remained_base) {
                        self.bids.push(bid); // Push the modified bid back into the heap
                    }

                    // Stop if the sell order is fully filled
                    if is_zero(&order.remained_base) {
                        break;
                    }
                }

                // Add the remaining sell order to the order book and update depth
                if !is_zero(&order.remained_base) {
                    self.asks.push(order.clone());
                }
            }
        }
        self.print_order_book();
        Ok(trades)
    }

    pub fn match_market_order(
        &mut self,
        mut order: TradeOrder,
    ) -> anyhow::Result<Vec<MatchedTrade>> {
        let mut trades = Vec::new();

        Self::print_order(&order);

        match order.side {
            OrderSide::Buy => {
                // Try to match the buy order with existing sell orders (asks)
                while let Some(mut ask) = self.asks.pop() {
                    // Calculate the trade amount
                    let trade_price = self.calculate_trade_price(&order, &ask, true)?;
                    let trade_amount = self.calculate_trade_amount(&order, &ask, &trade_price)?;

                    // Execute the trade
                    let trade =
                        self.execute_trade(&mut order, &mut ask, trade_amount, trade_price, true)?;
                    trades.push(trade);

                    // Remove the ask order if fully filled
                    if !is_zero(&ask.remained_base) {
                        self.asks.push(ask); // Push the modified ask back into the heap
                    }

                    // Stop if the buy order is fully filled
                    if is_zero(&order.remained_base) {
                        break;
                    }
                }

                // Cancel the MARKET order if not fully filled , we don't keep it in the order book
                if !is_zero(&order.remained_base) {
                    self.cancel_order(order.id)?;
                }
            }
            OrderSide::Sell => {
                // Try to match the sell order with existing buy orders (bids)
                while let Some(mut bid) = self.bids.pop() {
                    let trade_price = self.calculate_trade_price(&bid, &order, false)?;
                    // Calculate the trade amount
                    let trade_amount = self.calculate_trade_amount(&bid, &order, &trade_price)?;

                    // Execute the trade
                    let trade = self.execute_trade(
                        &mut bid,
                        &mut order,
                        trade_amount.clone(),
                        trade_price,
                        false,
                    )?;
                    trades.push(trade);

                    if !is_zero(&bid.remained_base) {
                        self.bids.push(bid); // Push the modified bid back into the heap
                    }

                    // Stop if the sell order is fully filled
                    if is_zero(&order.remained_base) {
                        break;
                    }
                }

                // Cancel the MARKET order if not fully filled , we don't keep it in the order book
                if !is_zero(&order.remained_base) {
                    self.cancel_order(order.id)?;
                }
            }
        }
        self.print_order_book();
        Ok(trades)
    }

    pub fn match_fok_order(&mut self, order: TradeOrder) -> anyhow::Result<Vec<MatchedTrade>> {
        let mut pop_orders: Vec<TradeOrder> = Vec::new();
        let mut is_fully_matched = false;
        let mut tem_order = order.clone();
        match order.side {
            OrderSide::Buy => {
                while let Some(ask) = self.asks.pop() {
                    if ask.price > order.price {
                        self.asks.push(ask);
                        break;
                    }
                    pop_orders.push(ask.clone());

                    let trade_price = self.calculate_trade_price(&tem_order, &ask, true)?;
                    let trade_amount =
                        self.calculate_trade_amount(&tem_order, &ask, &trade_price)?;

                    tem_order.remained_base = &tem_order.remained_base - &trade_amount;
                    tem_order.remained_quote =
                        &tem_order.remained_quote - &trade_amount * &trade_price;
                    if is_zero(&tem_order.remained_base) {
                        is_fully_matched = true;
                        break;
                    }
                }
            }
            OrderSide::Sell => {
                while let Some(bid) = self.bids.pop() {
                    if bid.price < order.price {
                        self.bids.push(bid);
                        break;
                    }
                    pop_orders.push(bid.clone());
                    let trade_price = self.calculate_trade_price(&bid, &tem_order, false)?;
                    let trade_amount =
                        self.calculate_trade_amount(&bid, &tem_order, &trade_price)?;

                    tem_order.remained_base = &tem_order.remained_base - &trade_amount;
                    tem_order.remained_quote =
                        &tem_order.remained_quote - &trade_amount * &trade_price;
                    if is_zero(&tem_order.remained_base) {
                        is_fully_matched = true;
                        break;
                    }
                }
            }
        }
        for order in pop_orders {
            self.asks.push(order);
        }
        if !is_fully_matched {
            self.cancel_order(order.id)?;
            return Err(anyhow::anyhow!("FOK order not fully matched"));
        } else {
            return self.match_limit_order(order);
        }
    }

    pub fn execute_trade(
        &mut self,
        buyer: &mut TradeOrder,
        seller: &mut TradeOrder,
        base_amount: BigDecimal,
        trade_price: BigDecimal,
        is_buyer_taker: bool,
    ) -> anyhow::Result<MatchedTrade> {
        // Calculate the fees for the buyer and seller
        let (buyer_fee, seller_fee) = match is_buyer_taker {
            true => (buyer.taker_fee.clone(), seller.maker_fee.clone()),
            false => (buyer.maker_fee.clone(), seller.taker_fee.clone()),
        };

        // Calculate the trade quote amount
        let trade_quote_amount = base_amount.clone() * trade_price.clone();

        // Execute the trade in a transaction
        let trade_data = self.persister.execute_limit_trade(
            is_buyer_taker,
            self.market_id.clone(),
            self.base_asset.clone(),
            self.quote_asset.clone(),
            buyer.user_id.clone(),
            seller.user_id.clone(),
            buyer.id.clone(),
            seller.id.clone(),
            trade_price.clone(),
            base_amount,
            trade_quote_amount,
            buyer_fee,
            seller_fee,
        )?;

        *buyer = self.persister.get_order(&buyer.id)?.unwrap().try_into()?;
        *seller = self.persister.get_order(&seller.id)?.unwrap().try_into()?;

        // Update the market price
        self.market_price = Some(trade_price);
        let is_liquidation = trade_data.is_liquidation.unwrap_or(false);
        self.handle_market_depth(&buyer);
        self.handle_market_depth(&seller);
        // Construct the trade object
        let trade = MatchedTrade {
            id: trade_data.id,
            timestamp: trade_data.timestamp,
            market_id: trade_data.market_id,
            price: trade_data.price,
            base_amount: trade_data.base_amount,
            quote_amount: trade_data.quote_amount,
            buyer_user_id: trade_data.buyer_user_id,
            buyer_order_id: trade_data.buyer_order_id,
            buyer_fee: trade_data.buyer_fee,
            seller_user_id: trade_data.seller_user_id,
            seller_order_id: trade_data.seller_order_id,
            seller_fee: trade_data.seller_fee,
            is_liquidation,
            taker_side: trade_data.taker_side.into(),
        };

        // Log trade execution
        Self::print_trade(&trade);
        // everything is done inside execute trade function so no need to call these functions her
        Ok(trade)
    }

    pub fn calculate_trade_price(
        &self,
        buyer: &TradeOrder,
        seller: &TradeOrder,
        is_buyer_taker: bool,
    ) -> anyhow::Result<BigDecimal> {
        match (buyer.order_type, seller.order_type) {
            // Market orders trade at last traded price if available
            (OrderType::Market, OrderType::Market) => {
                if let Some(last_price) = self.market_price.clone() {
                    Ok(last_price)
                } else {
                    Err(anyhow::anyhow!(
                        "No last traded price available for Market-Market order"
                    ))
                }
            }

            // Market order takes the price of the existing Limit order
            (OrderType::Market, OrderType::Limit) => Ok(seller.price.clone()),
            (OrderType::Limit, OrderType::Market) => Ok(buyer.price.clone()),

            // 1- If you place a buy limit order, and a seller is willing to sell at a price lower than your limit,
            // your order will execute at that lower price.
            // 2- In the same way, if you place a sell limit order, and a buyer is willing to buy at a price
            // higher than your limit price, your order will execute at that higher price.
            // Note: [buyer price is always the higher price(Or equal price) otherwise the order will not be matched]
            (OrderType::Limit, OrderType::Limit) => {
                if is_buyer_taker {
                    Ok(buyer.price.clone()) // best price for seller
                } else {
                    Ok(seller.price.clone()) // best price for buyer
                }
            }
        }
    }

    pub fn calculate_trade_amount(
        &self,
        buyer: &TradeOrder,
        seller: &TradeOrder,
        trade_price: &BigDecimal,
    ) -> anyhow::Result<BigDecimal> {
        if buyer.order_type == OrderType::Market {
            Ok((buyer.remained_quote.clone() / trade_price.clone())
                .with_prec(8)
                .min(seller.remained_base.clone()))
        } else {
            Ok(seller
                .remained_base
                .clone()
                .min(buyer.remained_base.clone()))
        }
    }
}

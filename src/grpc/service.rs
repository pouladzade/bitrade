use crate::market::market::Market;
use crate::models::order::{Order, OrderSide, OrderType};

use crate::grpc::spot::spot_service_server::SpotService;
use crate::grpc::spot::{
    AddOrderRequest, AddOrderResponse, CancelOrderRequest, CancelOrderResponse,
};
use rust_decimal::Decimal;
use std::str::FromStr;
use std::sync::Arc;
use tokio::sync::RwLock;
use tonic::{Request, Response, Status};

use super::helper::convert_trades;

#[derive(Debug)]
pub struct SpotServiceImpl {
    pub market: Arc<RwLock<Market>>,
}

#[tonic::async_trait]
impl SpotService for SpotServiceImpl {
    async fn add_order(
        &self,
        request: Request<AddOrderRequest>,
    ) -> Result<Response<AddOrderResponse>, Status> {
        let req = request.into_inner();
        let order = Order {
            id: req.id,
            base_asset: req.base_asset,
            quote_asset: req.quote_asset,
            market: req.market,
            order_type: OrderType::try_from(req.order_type).unwrap(),
            side: OrderSide::try_from(req.side).unwrap(),
            user_id: req.user_id,
            post_only: req.post_only,
            price: Decimal::from_str(&req.price).unwrap(),
            amount: Decimal::from_str(&req.amount).unwrap(),
            maker_fee: Decimal::from_str("0").unwrap(),
            taker_fee: Decimal::from_str("0").unwrap(),
            create_time: 0.0,
            remain: Decimal::from_str(&req.amount).unwrap(),
            frozen: Decimal::from_str("0").unwrap(),
            filled_base: Decimal::from_str("0").unwrap(),
            filled_quote: Decimal::from_str("0").unwrap(),
            filled_fee: Decimal::from_str("0").unwrap(),
            update_time: 0.0,
            partially_filled: false,
        };
        println!("gRPC Received order: {:?}", order);
        let order_book = self.market.write().await;
        let res = order_book.add_order(order);
        Ok(Response::new(AddOrderResponse {
            trades: convert_trades(res),
        }))
    }

    async fn cancel_order(
        &self,
        request: Request<CancelOrderRequest>,
    ) -> Result<Response<CancelOrderResponse>, Status> {
        let req = request.into_inner();
        let order_book = self.market.write().await;
        let success = order_book.cancel_order(req.order_id);

        Ok(Response::new(CancelOrderResponse { success }))
    }
}

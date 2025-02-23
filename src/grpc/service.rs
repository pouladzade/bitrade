use crate::market::market::Market;
use crate::models::order::{Order, OrderSide, OrderType};

use crate::grpc::spot::spot_service_server::SpotService;
use crate::grpc::spot::{
    AddOrderRequest, AddOrderResponse, CancelOrderRequest, CancelOrderResponse,
};
use anyhow::{Context, Result};
use rust_decimal::Decimal;
use std::str::FromStr;
use std::sync::Arc;
use tokio::sync::RwLock;
use tonic::{Request, Response, Status};

use super::helper::convert_trades;
use super::spot::{CancelAllOrdersRequest, CancelAllOrdersResponse};

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

        let order_type = OrderType::try_from(req.order_type.as_str())
            .map_err(|e| Status::invalid_argument(format!("Invalid order type: {}", e)))?;

        let side = OrderSide::try_from(req.side.as_str())
            .map_err(|e| Status::invalid_argument(format!("Invalid order side: {}", e)))?;

        let price = Decimal::from_str(&req.price)
            .context("Failed to parse price as Decimal")
            .map_err(|e| Status::invalid_argument(e.to_string()))?;

        let amount = Decimal::from_str(&req.amount)
            .context("Failed to parse amount as Decimal")
            .map_err(|e| Status::invalid_argument(e.to_string()))?;

        let maker_fee = Decimal::from_str("0")
            .context("Failed to parse maker_fee as Decimal")
            .map_err(|e| Status::internal(e.to_string()))?;

        let taker_fee = Decimal::from_str("0")
            .context("Failed to parse taker_fee as Decimal")
            .map_err(|e| Status::internal(e.to_string()))?;

        let order = Order {
            id: req.id,
            base_asset: req.base_asset,
            quote_asset: req.quote_asset,
            market: req.market,
            order_type,
            side,
            user_id: req.user_id,
            post_only: req.post_only,
            price,
            amount,
            maker_fee,
            taker_fee,
            create_time: 0.0,
            remain: amount,
            frozen: Decimal::ZERO,
            filled_base: Decimal::ZERO,
            filled_quote: Decimal::ZERO,
            filled_fee: Decimal::ZERO,
            update_time: 0.0,
            partially_filled: false,
        };

        let order_book = self.market.write().await;
        let res = order_book
            .add_order(order)
            .map_err(|e| Status::internal(e.to_string()))?;

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
        let success = order_book
            .cancel_order(req.order_id)
            .map_err(|e| Status::internal(e.to_string()))?;

        Ok(Response::new(CancelOrderResponse { success }))
    }

    #[allow(unused)]
    async fn cancel_all_orders(
        &self,
        request: Request<CancelAllOrdersRequest>,
    ) -> Result<Response<CancelAllOrdersResponse>, Status> {
        let req = request.into_inner();

        let order_book = self.market.write().await;
        let success = order_book
            .cancel_all_orders()
            .map_err(|e| Status::internal(e.to_string()))?;

        Ok(Response::new(CancelAllOrdersResponse { success }))
    }
}

use std::sync::Arc;

use super::helper::convert_trades;
use super::spot::{CancelAllOrdersRequest, CancelAllOrdersResponse};
use crate::grpc::spot::spot_service_server::SpotService;
use crate::grpc::spot::{
    AddOrderRequest, AddOrderResponse, CancelOrderRequest, CancelOrderResponse,
};
use crate::market::market::Market;
use crate::models::order::Order;
use anyhow::{Context, Result};
use tokio::sync::RwLock;
use tonic::{Request, Response, Status};

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

        let order = Order::try_from(req)
            .context("Failed to convert AddOrderRequest")
            .map_err(|e| Status::internal(e.to_string()))?;
        let order_book = self.market.write().await;
        let res = order_book
            .add_order(order)
            .map_err(|e| Status::internal(e.to_string()))?;

        Ok(Response::new(AddOrderResponse {
            trades: convert_trades(res.0),
            order_id: res.1,
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

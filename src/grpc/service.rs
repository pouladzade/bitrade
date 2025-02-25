use std::sync::Arc;

use super::helper::convert_trades;
use super::spot::{CancelAllOrdersRequest, CancelAllOrdersResponse};
use crate::grpc::spot::spot_service_server::SpotService;
use crate::grpc::spot::{
    AddOrderRequest, AddOrderResponse, CancelOrderRequest, CancelOrderResponse,
    CreateMarketRequest, CreateMarketResponse, StartMarketRequest, StartMarketResponse,
    StopMarketRequest, StopMarketResponse,
};
use crate::market::market_manager::MarketManager;
use crate::models::order::Order;
use anyhow::{Context, Result};
use tokio::sync::RwLock;
use tonic::{Request, Response, Status};

#[derive(Debug)]
pub struct SpotServiceImpl {
    pub market_manager: Arc<RwLock<MarketManager>>,
}

#[tonic::async_trait]
impl SpotService for SpotServiceImpl {
    async fn create_market(
        &self,
        request: Request<CreateMarketRequest>,
    ) -> Result<Response<CreateMarketResponse>, Status> {
        let req = request.into_inner();
        let market_id = req.market_id.clone();
        let pool_size = req.pool_size as usize;
        let market_manager = self.market_manager.write().await;
        market_manager
            .create_market(&market_id, pool_size)
            .context("Failed to create market")
            .map_err(|e| Status::internal(e.to_string()))?;

        Ok(Response::new(CreateMarketResponse {
            success: true,
            market_id,
        }))
    }

    async fn stop_market(
        &self,
        request: Request<StopMarketRequest>,
    ) -> Result<Response<StopMarketResponse>, Status> {
        let req = request.into_inner();
        let market_id = req.market_id.clone();
        let market_manager = self.market_manager.write().await;
        market_manager
            .stop_market(&market_id)
            .context("Failed to create market")
            .map_err(|e| Status::internal(e.to_string()))?;

        Ok(Response::new(StopMarketResponse {
            success: true,
            market_id,
        }))
    }

    async fn start_market(
        &self,
        request: Request<StartMarketRequest>,
    ) -> Result<Response<StartMarketResponse>, Status> {
        let req = request.into_inner();
        let market_id = req.market_id.clone();
        let market_manager = self.market_manager.write().await;
        market_manager
            .start_market(&market_id)
            .context("Failed to create market")
            .map_err(|e| Status::internal(e.to_string()))?;

        Ok(Response::new(StartMarketResponse {
            success: true,
            market_id,
        }))
    }

    async fn add_order(
        &self,
        request: Request<AddOrderRequest>,
    ) -> Result<Response<AddOrderResponse>, Status> {
        let req = request.into_inner();

        let order = Order::try_from(req)
            .context("Failed to convert AddOrderRequest")
            .map_err(|e| Status::internal(e.to_string()))?;
        let market_manager = self.market_manager.write().await;
        let res = market_manager
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
        let order_id = req.order_id.clone();
        let market_id = req.market_id.clone();
        let market_manager = self.market_manager.write().await;
        let success = market_manager
            .cancel_order(&req.order_id, req.market_id)
            .map_err(|e| Status::internal(e.to_string()))?;

        Ok(Response::new(CancelOrderResponse {
            success,
            order_id,
            market_id,
        }))
    }

    #[allow(unused)]
    async fn cancel_all_orders(
        &self,
        request: Request<CancelAllOrdersRequest>,
    ) -> Result<Response<CancelAllOrdersResponse>, Status> {
        let req = request.into_inner();
        let market_id = req.market_id.clone();
        let market_manager = self.market_manager.write().await;
        let success = market_manager
            .cancel_all_orders(&req.market_id)
            .context("Failed to cancel all orders")
            .map_err(|e| Status::internal(e.to_string()))?;

        Ok(Response::new(CancelAllOrdersResponse {
            success,
            market_id,
        }))
    }
}

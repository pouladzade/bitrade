use std::str::FromStr;
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
use crate::models::trade_order::TradeOrder;
use crate::utils;
use anyhow::{Context, Result};
use bigdecimal::BigDecimal;
use database::models::NewMarket;
use database::persistence::{self, ThreadSafePersistence};
use tokio::sync::RwLock;
use tonic::{Request, Response, Status};

pub struct SpotServiceImpl {
    pub market_manager: Arc<RwLock<MarketManager>>,
    pub persist: ThreadSafePersistence,
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

        self.persist
            .create_market(NewMarket {
                id: market_id.clone(),
                base_asset: req.base_asset.clone(),
                quote_asset: req.quote_asset.clone(),
                default_maker_fee: BigDecimal::from_str(&req.default_maker_fee)
                    .context("Failed to parse amount as Decimal")
                    .map_err(|e| Status::invalid_argument(e.to_string()))?,
                default_taker_fee: BigDecimal::from_str(&req.default_taker_fee)
                    .context("Failed to parse amount as Decimal")
                    .map_err(|e| Status::invalid_argument(e.to_string()))?,
                create_time: utils::get_utc_now_time_millisecond(),
                update_time: utils::get_utc_now_time_millisecond(),
            })
            .context("Failed to persist market")
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

        let order = TradeOrder::try_from(req)
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

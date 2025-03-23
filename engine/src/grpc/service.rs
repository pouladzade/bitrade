use super::helper::convert_trades;
use super::spot::WithdrawResponse;
use crate::grpc::spot::spot_service_server::SpotService;
use crate::grpc::spot::{
    AddOrderRequest, AddOrderResponse, CancelOrderRequest, CancelOrderResponse,
    CreateMarketRequest, CreateMarketResponse, StartMarketRequest, StartMarketResponse,
    StopMarketRequest, StopMarketResponse,
};
use crate::grpc::spot::{
    CancelAllOrdersRequest, CancelAllOrdersResponse, DepositRequest, DepositResponse,
    GetBalanceRequest, GetBalanceResponse, WithdrawRequest,
};
use crate::market::market_manager::MarketManager;
use crate::models::trade_order::TradeOrder;
use crate::validation::{validate_add_order_request, validate_create_market_request};
use crate::wallet::wallet_service::WalletService;
use anyhow::{Context, Result};
use bigdecimal::BigDecimal;
use database::provider::DatabaseProvider;
use std::str::FromStr;
use std::sync::Arc;
use tokio::sync::RwLock;
use tonic::{Request, Response, Status};

#[derive(Clone)]
pub struct SpotServiceImpl<P: DatabaseProvider + 'static> {
    pub market_manager: Arc<RwLock<MarketManager<P>>>,
    pub wallet_service: Arc<WalletService<P>>,
}

#[tonic::async_trait]
impl<P: DatabaseProvider + Send + Sync + 'static> SpotService for SpotServiceImpl<P> {
    async fn create_market(
        &self,
        request: Request<CreateMarketRequest>,
    ) -> Result<Response<CreateMarketResponse>, Status> {
        let req = request.into_inner();

        // Validate the request
        validate_create_market_request(&req)
            .map_err(|e| Status::invalid_argument(e.to_string()))?;

        let market_id = req.market_id.clone();
        let market_manager = self.market_manager.write().await;
        market_manager
            .create_market(
                market_id.clone(),
                req.base_asset,
                req.quote_asset,
                req.default_maker_fee,
                req.default_taker_fee,
            )
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
        let _ = market_manager
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

        // Validate the request
        validate_add_order_request(&req).map_err(|e| Status::invalid_argument(e.to_string()))?;

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
            .cancel_order(&req.market_id, req.order_id)
            .map_err(|e| Status::internal(e.to_string()))?;

        Ok(Response::new(CancelOrderResponse {
            success,
            order_id,
            market_id,
        }))
    }

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

    async fn deposit(
        &self,
        request: Request<DepositRequest>,
    ) -> Result<Response<DepositResponse>, Status> {
        let req = request.into_inner();

        let err_text = "Failed to convert amount from string";
        let res = self
            .wallet_service
            .deposit(
                &req.asset.clone(),
                BigDecimal::from_str(&req.amount)
                    .context(err_text)
                    .map_err(|e| Status::internal(e.to_string()))?,
                &req.user_id,
            )
            .context("Failed to deposit")
            .map_err(|e| Status::internal(e.to_string()))?;
        Ok(Response::new(DepositResponse {
            success: true,
            asset: res.asset,
            amount: res.available.to_string(),
            user_id: res.user_id,
        }))
    }

    async fn get_balance(
        &self,
        request: Request<GetBalanceRequest>,
    ) -> Result<Response<GetBalanceResponse>, Status> {
        let req = request.into_inner();

        let balance = self
            .wallet_service
            .get_balance(&req.asset, &req.user_id)
            .context("Failed to convert amount from string")
            .map_err(|e| Status::internal(e.to_string()))?;

        Ok(Response::new(GetBalanceResponse {
            user_id: req.user_id,
            asset: req.asset,
            amount: balance.to_string(),
        }))
    }

    async fn withdraw(
        &self,
        request: Request<WithdrawRequest>,
    ) -> Result<Response<WithdrawResponse>, Status> {
        let req = request.into_inner();

        let err_text = "Failed to convert amount from string";
        let res = self
            .wallet_service
            .withdraw(
                &req.asset.clone(),
                BigDecimal::from_str(&req.amount)
                    .context(err_text)
                    .map_err(|e| Status::internal(e.to_string()))?,
                &req.user_id,
            )
            .context("Failed to withdraw")
            .map_err(|e| Status::internal(e.to_string()))?;

        Ok(Response::new(WithdrawResponse {
            success: true,
            asset: res.asset,
            amount: res.available.to_string(),
            user_id: res.user_id,
        }))
    }
}

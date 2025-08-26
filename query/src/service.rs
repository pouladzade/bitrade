use crate::spot_query::{
    spot_query_service_server::SpotQueryService, GetFeeTreasuryRequest, GetFeeTreasuryResponse,
    GetMarketRequest, GetMarketResponse, GetMarketStatsRequest, GetMarketStatsResponse,
    GetOrderRequest, GetOrderResponse, GetUserTradesRequest, GetUserTradesResponse,
    GetWalletRequest, GetWalletResponse, ListMarketsRequest, ListMarketsResponse,
    ListOrdersRequest, ListOrdersResponse, ListTradesRequest, ListTradesResponse,
    ListWalletsRequest, ListWalletsResponse, PaginationResponse,
};
use anyhow::Result;
use common::db::pagination::Pagination;
use database::{
    filters::{OrderFilter, TradeFilter, WalletFilter},
    provider::{
        FeeTreasuryDatabaseReader, MarketDatabaseReader, MarketStatDatabaseReader,
        OrderDatabaseReader, TradeDatabaseReader, WalletDatabaseReader,
    },
};
use tonic::{Request, Response, Status};

pub struct SpotQueryServiceImp<R> {
    pub repository: R,
}

impl<R> SpotQueryServiceImp<R> {
    pub fn new(repository: R) -> Self {
        Self { repository }
    }
}

#[tonic::async_trait]
impl<R> SpotQueryService for SpotQueryServiceImp<R>
where
    R: MarketDatabaseReader
        + OrderDatabaseReader
        + TradeDatabaseReader
        + WalletDatabaseReader
        + MarketStatDatabaseReader
        + FeeTreasuryDatabaseReader
        + Send
        + Sync
        + 'static,
{
    async fn get_market(
        &self,
        request: Request<GetMarketRequest>,
    ) -> Result<Response<GetMarketResponse>, Status> {
        let market_id = &request.into_inner().market_id;
        let market = self
            .repository
            .get_market(market_id)
            .map_err(|e| Status::internal(e.to_string()))?
            .ok_or_else(|| Status::not_found("Market not found"))?;

        Ok(Response::new(GetMarketResponse {
            market: Some(market.into()),
        }))
    }

    async fn list_markets(
        &self,
        _request: Request<ListMarketsRequest>,
    ) -> Result<Response<ListMarketsResponse>, Status> {
        let markets = self
            .repository
            .list_markets()
            .map_err(|e| Status::internal(e.to_string()))?;

        Ok(Response::new(ListMarketsResponse {
            markets: markets.into_iter().map(|m| m.into()).collect(),
        }))
    }

    async fn get_order(
        &self,
        request: Request<GetOrderRequest>,
    ) -> Result<Response<GetOrderResponse>, Status> {
        let order_id = &request.into_inner().order_id;
        let order = self
            .repository
            .get_order(order_id)
            .map_err(|e| Status::internal(e.to_string()))?
            .ok_or_else(|| Status::not_found("Order not found"))?;

        Ok(Response::new(GetOrderResponse {
            order: Some(order.into()),
        }))
    }

    async fn list_orders(
        &self,
        request: Request<ListOrdersRequest>,
    ) -> Result<Response<ListOrdersResponse>, Status> {
        let req = request.into_inner();
        let filter = OrderFilter::from(req.filter.unwrap());
        let pagination = Pagination::from(req.pagination.unwrap());

        let paginated = self
            .repository
            .list_orders(filter, Some(pagination))
            .map_err(|e| Status::internal(e.to_string()))?;

        Ok(Response::new(ListOrdersResponse {
            orders: paginated.items.into_iter().map(|o| o.into()).collect(),
            pagination: Some(PaginationResponse {
                total_count: paginated.total_count,
                has_more: paginated.has_more,
                next_offset: paginated.next_offset.unwrap_or(0),
            }),
        }))
    }

    async fn list_trades(
        &self,
        request: Request<ListTradesRequest>,
    ) -> Result<Response<ListTradesResponse>, Status> {
        let req = request.into_inner();
        let filter = TradeFilter::from(req.filter.unwrap_or_default());
        let pagination = Pagination::from(req.pagination.unwrap_or_default());

        let paginated = self
            .repository
            .list_trades(filter, Some(pagination))
            .map_err(|e| Status::internal(e.to_string()))?;

        Ok(Response::new(ListTradesResponse {
            trades: paginated.items.into_iter().map(|t| t.into()).collect(),
            pagination: Some(PaginationResponse {
                total_count: paginated.total_count,
                has_more: paginated.has_more,
                next_offset: paginated.next_offset.unwrap_or(0),
            }),
        }))
    }

    async fn get_wallet(
        &self,
        request: Request<GetWalletRequest>,
    ) -> Result<Response<GetWalletResponse>, Status> {
        let req = request.into_inner();
        let wallet = self
            .repository
            .get_wallet(&req.user_id, &req.asset)
            .map_err(|e| Status::internal(e.to_string()))?
            .ok_or_else(|| Status::not_found("Wallet not found"))?;

        Ok(Response::new(GetWalletResponse {
            wallet: Some(wallet.into()),
        }))
    }

    async fn list_wallets(
        &self,
        request: Request<ListWalletsRequest>,
    ) -> Result<Response<ListWalletsResponse>, Status> {
        let req = request.into_inner();
        let pagination = req.pagination.map(|p| Pagination {
            limit: Some(p.limit as i64),
            offset: Some(p.offset as i64),
            order_by: Some(p.order_by),
            order_direction: Some(p.order_direction),
        });
        let filter = req.filter.unwrap_or_default();
        let paginated_wallets = self
            .repository
            .list_wallets(
                WalletFilter {
                    user_id: filter.user_id,
                    asset: filter.asset,
                    ..Default::default()
                },
                pagination,
            )
            .map_err(|e| Status::internal(e.to_string()))?;

        Ok(Response::new(ListWalletsResponse {
            wallets: paginated_wallets
                .items
                .into_iter()
                .map(|w| w.into())
                .collect(),
            pagination: Some(PaginationResponse {
                total_count: paginated_wallets.total_count,
                has_more: paginated_wallets.has_more,
                next_offset: paginated_wallets.next_offset.unwrap_or(0),
            }),
        }))
    }

    async fn get_market_stats(
        &self,
        request: Request<GetMarketStatsRequest>,
    ) -> Result<Response<GetMarketStatsResponse>, Status> {
        let market_id = &request.into_inner().market_id;
        let stats = self
            .repository
            .get_market_stats(market_id)
            .map_err(|e| Status::internal(e.to_string()))?
            .ok_or_else(|| Status::not_found("Market stats not found"))?;

        Ok(Response::new(GetMarketStatsResponse {
            stats: Some(stats.into()),
        }))
    }

    async fn get_fee_treasury(
        &self,
        request: Request<GetFeeTreasuryRequest>,
    ) -> Result<Response<GetFeeTreasuryResponse>, Status> {
        let req = request.into_inner();
        let treasury = self
            .repository
            .get_fee_treasury(&req.market_id)
            .map_err(|e| Status::internal(e.to_string()))?
            .ok_or_else(|| Status::not_found("Fee treasury not found"))?;

        Ok(Response::new(GetFeeTreasuryResponse {
            treasury: Some(treasury.into()),
        }))
    }

    async fn get_user_trades(
        &self,
        request: Request<GetUserTradesRequest>,
    ) -> Result<Response<GetUserTradesResponse>, Status> {
        let req = request.into_inner();
        let pagination = Pagination::from(req.pagination.unwrap_or_default());

        // Create a filter for user trades
        let filter = TradeFilter::new()
            .buyer_user_id(Some(req.user_id.clone()))
            .seller_user_id(Some(req.user_id.clone()))
            .start_time(if req.start_time > 0 {
                Some(req.start_time)
            } else {
                None
            })
            .end_time(if req.end_time > 0 {
                Some(req.end_time)
            } else {
                None
            });

        let paginated_trades = self
            .repository
            .list_trades(filter, Some(pagination))
            .map_err(|e| Status::internal(e.to_string()))?;

        Ok(Response::new(GetUserTradesResponse {
            trades: paginated_trades
                .items
                .into_iter()
                .map(|t| t.into())
                .collect(),
            pagination: Some(PaginationResponse {
                total_count: paginated_trades.total_count,
                has_more: paginated_trades.has_more,
                next_offset: paginated_trades.next_offset.unwrap_or(0),
            }),
        }))
    }
}

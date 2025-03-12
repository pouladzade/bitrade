pub struct QueryServiceImpl {
    query_repo: Box<dyn QueryRepository + Send + Sync>,
}

impl QueryServiceImpl {
    pub fn new(pool: DbPool) -> Self {
        Self {
            query_repo: Box::new(QueryRepositoryImpl::new(pool)),
        }
    }
}

#[tonic::async_trait]
impl QueryService for QueryServiceImpl {
    async fn get_user_orders(
        &self,
        request: Request<GetUserOrdersRequest>,
    ) -> Result<Response<GetUserOrdersResponse>, Status> {
        let req = request.into_inner();
        
        // Decode cursor if provided
        let cursor = if !req.page_token.is_empty() {
            Some(self.query_repo
                .decode_cursor::<OrderCursor>(&req.page_token)
                .map_err(|e| Status::invalid_argument(format!("Invalid cursor: {}", e)))?)
        } else {
            None
        };

        let pagination = PaginationOptions {
            limit: req.page_size.min(100) as i64, // Enforce maximum page size
            cursor,
        };

        let (orders, next_page_token) = self.query_repo
            .get_user_orders(&req.user_id, pagination)
            .map_err(|e| Status::internal(e.to_string()))?;

        Ok(Response::new(GetUserOrdersResponse {
            orders: orders.into_iter().map(Into::into).collect(),
            next_page_token: next_page_token.unwrap_or_default(),
        }))
    }
    
    async fn get_market_trades(
        &self,
        request: Request<GetMarketTradesRequest>,
    ) -> Result<Response<GetMarketTradesResponse>, Status> {
        let req = request.into_inner();
        
        // Decode cursor if provided
        let cursor = if !req.page_token.is_empty() {
            Some(self.query_repo
                .decode_cursor::<TradeCursor>(&req.page_token)
                .map_err(|e| Status::invalid_argument(format!("Invalid cursor: {}", e)))?)
        } else {
            None
        };

        let pagination = PaginationOptions {
            limit: req.page_size.min(100) as i64,
            cursor,
        };

        let (trades, next_page_token) = self.query_repo
            .get_market_trades(&req.market_id, pagination)
            .map_err(|e| Status::internal(e.to_string()))?;

        Ok(Response::new(GetMarketTradesResponse {
            trades: trades.into_iter().map(Into::into).collect(),
            next_page_token: next_page_token.unwrap_or_default(),
        }))
    }

} 
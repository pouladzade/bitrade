use diesel::prelude::*;
use diesel::r2d2::{ConnectionManager, Pool};
use tonic::{Request, Response, Status};
use base64::{Engine as _, engine::general_purpose::STANDARD as BASE64};

use crate::{models, proto, schema};
use crate::proto::query_service_server::QueryService;

pub struct QueryServiceImpl {
    pool: Pool<ConnectionManager<PgConnection>>,
}

impl QueryServiceImpl {
    pub fn new(pool: Pool<ConnectionManager<PgConnection>>) -> Self {
        Self { pool }
    }
}

#[tonic::async_trait]
impl QueryService for QueryServiceImpl {
    async fn get_market(
        &self,
        request: Request<proto::GetMarketRequest>,
    ) -> Result<Response<proto::GetMarketResponse>, Status> {
        let conn = self.pool.get()
            .map_err(|e| Status::internal(format!("Database error: {}", e)))?;

        let market_id = request.into_inner().market_id;

        let market = schema::markets::table
            .find(market_id)
            .first::<models::Market>(&mut conn)
            .optional()
            .map_err(|e| Status::internal(format!("Database error: {}", e)))?
            .ok_or_else(|| Status::not_found("Market not found"))?;

        Ok(Response::new(proto::GetMarketResponse {
            market: Some(market.into()),
        }))
    }

    async fn list_markets(
        &self,
        request: Request<proto::ListMarketsRequest>,
    ) -> Result<Response<proto::ListMarketsResponse>, Status> {
        let conn = self.pool.get()
            .map_err(|e| Status::internal(format!("Database error: {}", e)))?;

        let req = request.into_inner();
        let limit = req.page_size.min(100) as i64;

        // Decode the page token if provided
        let last_seen_id = if !req.page_token.is_empty() {
            Some(String::from_utf8(
                BASE64.decode(req.page_token)
                    .map_err(|_| Status::invalid_argument("Invalid page token"))?
            ).map_err(|_| Status::invalid_argument("Invalid page token"))?)
        } else {
            None
        };

        // Build query
        let mut query = schema::markets::table
            .into_boxed();

        // Apply status filter if provided
        if !req.status.is_empty() {
            query = query.filter(schema::markets::status.eq(req.status));
        }

        // Apply pagination
        if let Some(last_id) = last_seen_id {
            query = query.filter(schema::markets::id.gt(last_id));
        }

        // Execute query
        let markets: Vec<models::Market> = query
            .order_by(schema::markets::id.asc())
            .limit(limit)
            .load::<models::Market>(&mut conn)
            .map_err(|e| Status::internal(format!("Database error: {}", e)))?;

        // Generate next page token
        let next_page_token = if markets.len() == limit as usize {
            if let Some(last_market) = markets.last() {
                BASE64.encode(last_market.id.as_bytes())
            } else {
                String::new()
            }
        } else {
            String::new()
        };

        Ok(Response::new(proto::ListMarketsResponse {
            markets: markets.into_iter().map(Into::into).collect(),
            next_page_token,
        }))
    }

  
} 
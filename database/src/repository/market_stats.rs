use crate::models::models::*;

use crate::models::schema::*;
use anyhow::Result;
use bigdecimal::BigDecimal;
use diesel::prelude::*;

use super::Repository;

impl Repository {
    // Market stats operations
    pub fn update_market_stats(
        &self,
        market_id: &str,
        high_24h: BigDecimal,
        low_24h: BigDecimal,
        volume_24h: BigDecimal,
        price_change_24h: BigDecimal,
        last_price: BigDecimal,
    ) -> Result<MarketStat> {
        let conn = &mut self.get_conn()?;

        let current_time = chrono::Utc::now().timestamp_millis();

        // Check if stats exist
        let stats_option = market_stats::table
            .find(market_id)
            .first::<MarketStat>(conn)
            .optional()?;

        if let Some(_) = stats_option {
            // Update existing stats
            let result = diesel::update(market_stats::table.find(market_id))
                .set((
                    market_stats::high_24h.eq(high_24h),
                    market_stats::low_24h.eq(low_24h),
                    market_stats::volume_24h.eq(volume_24h),
                    market_stats::price_change_24h.eq(price_change_24h),
                    market_stats::last_price.eq(last_price),
                    market_stats::last_update_time.eq(current_time),
                ))
                .get_result(conn)?;

            Ok(result)
        } else {
            // Create new stats
            let new_stats = NewMarketStat {
                market_id: market_id.to_string(),
                high_24h,
                low_24h,
                volume_24h,
                price_change_24h,
                last_price,
                last_update_time: current_time,
            };

            let result = diesel::insert_into(market_stats::table)
                .values(&new_stats)
                .get_result(conn)?;

            Ok(result)
        }
    }

    pub fn get_market_stats(&self, market_id: &str) -> Result<Option<MarketStat>> {
        let conn = &mut self.get_conn()?;

        let result = market_stats::table.find(market_id).first(conn).optional()?;

        Ok(result)
    }
}

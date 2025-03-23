use common::db::pagination::Pagination;
use database::filters::{OrderFilter, TradeFilter};
use database::models::models::{FeeTreasury, Market, MarketStat, Order, Trade, Wallet};

use crate::spot_query::{
    PaginationRequest, PaginationResponse, ProtoFeeTreasury, ProtoMarket, ProtoMarketStats,
    ProtoOrder, ProtoOrderFilter, ProtoTrade, ProtoTradeFilter, ProtoWallet,
};

impl From<Market> for ProtoMarket {
    fn from(m: Market) -> Self {
        ProtoMarket {
            id: m.id,
            base_asset: m.base_asset,
            quote_asset: m.quote_asset,
            default_maker_fee: m.default_maker_fee.to_string(),
            default_taker_fee: m.default_taker_fee.to_string(),
            create_time: m.create_time,
            update_time: m.update_time,
            status: m.status,
            min_base_amount: m.min_base_amount.to_string(),
            min_quote_amount: m.min_quote_amount.to_string(),
            price_precision: m.price_precision,
            amount_precision: m.amount_precision,
        }
    }
}

impl From<Order> for ProtoOrder {
    fn from(o: Order) -> Self {
        ProtoOrder {
            id: o.id,
            market_id: o.market_id,
            user_id: o.user_id,
            order_type: o.order_type,
            side: o.side,
            price: o.price.to_string(),
            base_amount: o.base_amount.to_string(),
            quote_amount: o.quote_amount.to_string(),
            maker_fee: o.maker_fee.to_string(),
            taker_fee: o.taker_fee.to_string(),
            create_time: o.create_time,
            remained_base: o.remained_base.to_string(),
            remained_quote: o.remained_quote.to_string(),
            filled_base: o.filled_base.to_string(),
            filled_quote: o.filled_quote.to_string(),
            filled_fee: o.filled_fee.to_string(),
            update_time: o.update_time,
            status: o.status,
            client_order_id: o.client_order_id.unwrap_or_default(),
            post_only: o.post_only.unwrap_or(false),
            time_in_force: o.time_in_force.unwrap_or_default(),
            expires_at: o.expires_at.unwrap_or(0),
        }
    }
}

impl From<Trade> for ProtoTrade {
    fn from(t: Trade) -> Self {
        ProtoTrade {
            id: t.id,
            timestamp: t.timestamp,
            market_id: t.market_id,
            price: t.price.to_string(),
            base_amount: t.base_amount.to_string(),
            quote_amount: t.quote_amount.to_string(),
            buyer_user_id: t.buyer_user_id,
            buyer_order_id: t.buyer_order_id,
            buyer_fee: t.buyer_fee.to_string(),
            seller_user_id: t.seller_user_id,
            seller_order_id: t.seller_order_id,
            seller_fee: t.seller_fee.to_string(),
            taker_side: t.taker_side,
            is_liquidation: t.is_liquidation.unwrap_or(false),
        }
    }
}

impl From<Wallet> for ProtoWallet {
    fn from(w: Wallet) -> Self {
        ProtoWallet {
            user_id: w.user_id,
            asset: w.asset,
            available: w.available.to_string(),
            locked: w.locked.to_string(),
            reserved: w.reserved.to_string(),
            total_deposited: w.total_deposited.to_string(),
            total_withdrawn: w.total_withdrawn.to_string(),
            update_time: w.update_time,
        }
    }
}

impl From<MarketStat> for ProtoMarketStats {
    fn from(s: MarketStat) -> Self {
        ProtoMarketStats {
            market_id: s.market_id,
            high_24h: s.high_24h.to_string(),
            low_24h: s.low_24h.to_string(),
            volume_24h: s.volume_24h.to_string(),
            price_change_24h: s.price_change_24h.to_string(),
            last_price: s.last_price.to_string(),
            last_update_time: s.last_update_time,
        }
    }
}

impl From<FeeTreasury> for ProtoFeeTreasury {
    fn from(f: FeeTreasury) -> Self {
        ProtoFeeTreasury {
            treasury_address: f.treasury_address,
            market_id: f.market_id,
            asset: f.asset,
            collected_amount: f.collected_amount.to_string(),
            last_update_time: f.last_update_time,
        }
    }
}

impl From<PaginationRequest> for Pagination {
    fn from(p: PaginationRequest) -> Self {
        Pagination {
            limit: Some(p.limit),
            offset: Some(p.offset),
            order_by: Some(p.order_by.to_string()),
            order_direction: Some(p.order_direction.to_string()),
        }
    }
}

impl From<ProtoOrderFilter> for OrderFilter {
    fn from(f: ProtoOrderFilter) -> Self {
        OrderFilter::new()
            .user_id(f.user_id)
            .market_id(f.market_id)
            .order_id(f.order_id)
            .side(f.side)
            .status(f.status)
            .order_type(f.order_type)
    }
}

impl From<ProtoTradeFilter> for TradeFilter {
    fn from(f: ProtoTradeFilter) -> Self {
        TradeFilter::new()
            .market_id(f.market_id)
            .buyer_order_id(f.buyer_order_id)
            .seller_order_id(f.seller_order_id)
            .buyer_user_id(f.buyer_user_id)
            .seller_user_id(f.seller_user_id)
            .taker_side(f.taker_side)
            .is_liquidation(f.is_liquidation)
            .start_time(f.start_time)
            .end_time(f.end_time)
    }
}

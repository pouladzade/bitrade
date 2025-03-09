// @generated automatically by Diesel CLI.

diesel::table! {
    balances (user_id, asset) {
        #[max_length = 50]
        user_id -> Varchar,
        #[max_length = 20]
        asset -> Varchar,
        available -> Numeric,
        locked -> Numeric,
        update_time -> Int8,
        reserved -> Numeric,
        total_deposited -> Numeric,
        total_withdrawn -> Numeric,
    }
}

diesel::table! {
    fee_treasury (id) {
        id -> Int4,
        #[max_length = 100]
        treasury_address -> Varchar,
        #[max_length = 50]
        market_id -> Varchar,
        #[max_length = 20]
        asset -> Varchar,
        collected_amount -> Numeric,
        last_update_time -> Int8,
    }
}

diesel::table! {
    market_stats (market_id) {
        #[max_length = 50]
        market_id -> Varchar,
        high_24h -> Numeric,
        low_24h -> Numeric,
        volume_24h -> Numeric,
        price_change_24h -> Numeric,
        last_price -> Numeric,
        last_update_time -> Int8,
    }
}

diesel::table! {
    markets (id) {
        #[max_length = 50]
        id -> Varchar,
        #[max_length = 20]
        base_asset -> Varchar,
        #[max_length = 20]
        quote_asset -> Varchar,
        default_maker_fee -> Numeric,
        default_taker_fee -> Numeric,
        create_time -> Int8,
        update_time -> Int8,
        #[max_length = 20]
        status -> Varchar,
        min_base_amount -> Numeric,
        min_quote_amount -> Numeric,
        price_precision -> Int4,
        amount_precision -> Int4,
    }
}

diesel::table! {
    orders (id) {
        #[max_length = 50]
        id -> Varchar,
        #[max_length = 50]
        market_id -> Varchar,
        #[max_length = 50]
        user_id -> Varchar,
        #[max_length = 20]
        order_type -> Varchar,
        #[max_length = 10]
        side -> Varchar,
        price -> Numeric,
        base_amount -> Numeric,
        quote_amount -> Numeric,
        maker_fee -> Numeric,
        taker_fee -> Numeric,
        create_time -> Int8,
        remained_base -> Numeric,
        remained_quote -> Numeric,
        filled_base -> Numeric,
        filled_quote -> Numeric,
        filled_fee -> Numeric,
        update_time -> Int8,
        #[max_length = 20]
        status -> Varchar,
        #[max_length = 50]
        client_order_id -> Nullable<Varchar>,
        post_only -> Nullable<Bool>,
        #[max_length = 10]
        time_in_force -> Nullable<Varchar>,
        expires_at -> Nullable<Int8>,
    }
}

diesel::table! {
    trades (id) {
        #[max_length = 50]
        id -> Varchar,
        timestamp -> Int8,
        #[max_length = 50]
        market_id -> Varchar,
        price -> Numeric,
        base_amount -> Numeric,
        quote_amount -> Numeric,
        #[max_length = 50]
        buyer_user_id -> Varchar,
        #[max_length = 50]
        buyer_order_id -> Varchar,
        buyer_fee -> Numeric,
        #[max_length = 50]
        seller_user_id -> Varchar,
        #[max_length = 50]
        seller_order_id -> Varchar,
        seller_fee -> Numeric,
        #[max_length = 10]
        taker_side -> Varchar,
        is_liquidation -> Nullable<Bool>,
    }
}

diesel::joinable!(fee_treasury -> markets (market_id));
diesel::joinable!(market_stats -> markets (market_id));
diesel::joinable!(orders -> markets (market_id));
diesel::joinable!(trades -> markets (market_id));

diesel::allow_tables_to_appear_in_same_query!(
    balances,
    fee_treasury,
    market_stats,
    markets,
    orders,
    trades,
);

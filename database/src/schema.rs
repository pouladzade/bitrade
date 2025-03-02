// @generated automatically by Diesel CLI.

diesel::table! {
    balances (user_id, asset) {
        #[max_length = 50]
        user_id -> Varchar,
        #[max_length = 20]
        asset -> Varchar,
        available -> Numeric,
        frozen -> Numeric,
        update_time -> Int8,
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
        amount -> Numeric,
        maker_fee -> Numeric,
        taker_fee -> Numeric,
        create_time -> Int8,
        remain -> Numeric,
        filled_base -> Numeric,
        filled_quote -> Numeric,
        filled_fee -> Numeric,
        update_time -> Int8,
        #[max_length = 20]
        status -> Varchar,
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
        amount -> Numeric,
        quote_amount -> Numeric,
        #[max_length = 50]
        taker_user_id -> Varchar,
        #[max_length = 50]
        taker_order_id -> Varchar,
        taker_fee -> Numeric,
        #[max_length = 50]
        maker_user_id -> Varchar,
        #[max_length = 50]
        maker_order_id -> Varchar,
        maker_fee -> Numeric,
    }
}

diesel::joinable!(market_stats -> markets (market_id));
diesel::joinable!(orders -> markets (market_id));
diesel::joinable!(trades -> markets (market_id));

diesel::allow_tables_to_appear_in_same_query!(
    balances,
    market_stats,
    markets,
    orders,
    trades,
);

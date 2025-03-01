pub struct Market {
    pub id: String,
    pub base_asset: String,
    pub quote_asset: String,

    pub default_maker_fee: BigDecimal,
    pub default_taker_fee: BigDecimal,

    pub create_time: i64,
    pub update_time: i64,
}

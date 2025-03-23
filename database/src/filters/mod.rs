#[derive(Default, Clone)]
pub struct OrderFilter {
    pub user_id: Option<String>,
    pub market_id: Option<String>,
    pub order_id: Option<String>,
    pub side: Option<String>,
    pub status: Option<String>,
    pub order_type: Option<String>,
}

impl OrderFilter {
    pub fn new() -> Self {
        Self::default()
    }

    pub fn user_id(mut self, user_id: Option<String>) -> Self {
        self.user_id = user_id;
        self
    }

    pub fn market_id(mut self, market_id: Option<String>) -> Self {
        self.market_id = market_id;
        self
    }

    pub fn order_id(mut self, order_id: Option<String>) -> Self {
        self.order_id = order_id;
        self
    }

    pub fn side(mut self, side: Option<String>) -> Self {
        self.side = side;
        self
    }

    pub fn status(mut self, status: Option<String>) -> Self {
        self.status = status;
        self
    }

    pub fn order_type(mut self, order_type: Option<String>) -> Self {
        self.order_type = order_type;
        self
    }
}

#[derive(Default, Clone)]
pub struct TradeFilter {
    pub market_id: Option<String>,
    pub buyer_order_id: Option<String>,
    pub seller_order_id: Option<String>,
    pub buyer_user_id: Option<String>,
    pub seller_user_id: Option<String>,
    pub taker_side: Option<String>,
    pub is_liquidation: Option<bool>,
    pub start_time: Option<i64>,
    pub end_time: Option<i64>,
}

impl TradeFilter {
    pub fn new() -> Self {
        Self::default()
    }

    pub fn market_id(mut self, market_id: Option<String>) -> Self {
        self.market_id = market_id;
        self
    }

    pub fn buyer_order_id(mut self, buyer_order_id: Option<String>) -> Self {
        self.buyer_order_id = buyer_order_id;
        self
    }

    pub fn seller_order_id(mut self, seller_order_id: Option<String>) -> Self {
        self.seller_order_id = seller_order_id;
        self
    }

    pub fn buyer_user_id(mut self, buyer_user_id: Option<String>) -> Self {
        self.buyer_user_id = buyer_user_id;
        self
    }

    pub fn seller_user_id(mut self, seller_user_id: Option<String>) -> Self {
        self.seller_user_id = seller_user_id;
        self
    }

    pub fn taker_side(mut self, taker_side: Option<String>) -> Self {
        self.taker_side = taker_side;
        self
    }

    pub fn is_liquidation(mut self, is_liquidation: Option<bool>) -> Self {
        self.is_liquidation = is_liquidation;
        self
    }

    pub fn start_time(mut self, start_time: Option<i64>) -> Self {
        self.start_time = start_time;
        self
    }

    pub fn end_time(mut self, end_time: Option<i64>) -> Self {
        self.end_time = end_time;
        self
    }
}

#[derive(Default, Clone)]
pub struct MarketFilter {
    pub market_id: Option<String>,
    pub market_name: Option<String>,
    pub market_symbol: Option<String>,
}

impl MarketFilter {
    pub fn new() -> Self {
        Self::default()
    }

    pub fn market_id(mut self, market_id: Option<String>) -> Self {
        self.market_id = market_id;
        self
    }

    pub fn market_name(mut self, market_name: Option<String>) -> Self {
        self.market_name = market_name;
        self
    }

    pub fn market_symbol(mut self, market_symbol: Option<String>) -> Self {
        self.market_symbol = market_symbol;
        self
    }
}

#[derive(Default, Clone)]
pub struct WalletFilter {
    pub user_id: Option<String>,
    pub asset: Option<String>,
}

impl WalletFilter {
    pub fn new() -> Self {
        Self::default()
    }

    pub fn user_id(mut self, user_id: Option<String>) -> Self {
        self.user_id = user_id;
        self
    }

    pub fn asset(mut self, asset: Option<String>) -> Self {
        self.asset = asset;
        self
    }
}

#[derive(Default, Clone)]
pub struct FeeTreasuryFilter {
    pub market_id: Option<String>,
    pub asset: Option<String>,
}

impl FeeTreasuryFilter {
    pub fn new() -> Self {
        Self::default()
    }

    pub fn market_id(mut self, market_id: Option<String>) -> Self {
        self.market_id = market_id;
        self
    }

    pub fn asset(mut self, asset: Option<String>) -> Self {
        self.asset = asset;
        self
    }
}

#[derive(Default, Clone)]
pub struct MarketStatFilter {
    pub market_id: Option<String>,
    pub start_time: Option<i64>,
    pub end_time: Option<i64>,
}

impl MarketStatFilter {
    pub fn new() -> Self {
        Self::default()
    }

    pub fn market_id(mut self, market_id: Option<String>) -> Self {
        self.market_id = market_id;
        self
    }

    pub fn start_time(mut self, start_time: Option<i64>) -> Self {
        self.start_time = start_time;
        self
    }

    pub fn end_time(mut self, end_time: Option<i64>) -> Self {
        self.end_time = end_time;
        self
    }
}

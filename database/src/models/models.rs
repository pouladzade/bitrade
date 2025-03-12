use bigdecimal::BigDecimal;
use diesel::prelude::*;
use serde::{Deserialize, Serialize};

use crate::models::schema::*;

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub enum OrderType {
    Limit,
    Market,
}

impl OrderType {
    pub fn as_str(&self) -> &'static str {
        match self {
            OrderType::Limit => "LIMIT",
            OrderType::Market => "MARKET",
        }
    }

    pub fn from_str(s: &str) -> Result<Self, String> {
        match s.to_uppercase().as_str() {
            "LIMIT" => Ok(OrderType::Limit),
            "MARKET" => Ok(OrderType::Market),
            _ => Err(format!("Unknown order type: {}", s)),
        }
    }
}

// Represents the OrderSide enum in your application
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub enum OrderSide {
    Buy,
    Sell,
}

impl OrderSide {
    pub fn as_str(&self) -> &'static str {
        match self {
            OrderSide::Buy => "BUY",
            OrderSide::Sell => "SELL",
        }
    }

    pub fn from_str(s: &str) -> Result<Self, String> {
        match s.to_uppercase().as_str() {
            "BUY" => Ok(OrderSide::Buy),
            "SELL" => Ok(OrderSide::Sell),
            _ => Err(format!("Unknown order side: {}", s)),
        }
    }
}

// Represents the MarketRole enum in your application
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub enum MarketRole {
    Maker,
    Taker,
}

impl MarketRole {
    pub fn as_str(&self) -> &'static str {
        match self {
            MarketRole::Maker => "MAKER",
            MarketRole::Taker => "TAKER",
        }
    }

    pub fn from_str(s: &str) -> Result<Self, String> {
        match s.to_uppercase().as_str() {
            "MAKER" => Ok(MarketRole::Maker),
            "TAKER" => Ok(MarketRole::Taker),
            _ => Err(format!("Unknown market role: {}", s)),
        }
    }
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub enum OrderStatus {
    Open,
    Filled,
    Canceled,
    Rejected,
    PartiallyFilled,
}

impl OrderStatus {
    pub fn as_str(&self) -> &'static str {
        match self {
            OrderStatus::Open => "OPEN",
            OrderStatus::Filled => "FILLED",
            OrderStatus::Canceled => "CANCELED",
            OrderStatus::Rejected => "REJECTED",
            OrderStatus::PartiallyFilled => "PARTIALLY_FILLED",
        }
    }

    pub fn from_str(s: &str) -> Result<Self, String> {
        match s.to_uppercase().as_str() {
            "OPEN" => Ok(OrderStatus::Open),
            "FILLED" => Ok(OrderStatus::Filled),
            "CANCELED" => Ok(OrderStatus::Canceled),
            "REJECTED" => Ok(OrderStatus::Rejected),
            "PARTIALLY_FILLED" => Ok(OrderStatus::PartiallyFilled),
            _ => Err(format!("Unknown order status: {}", s)),
        }
    }
}

impl TryFrom<&str> for OrderStatus {
    type Error = String;

    fn try_from(value: &str) -> Result<Self, Self::Error> {
        Self::from_str(value)
    }
}

// Add TimeInForce enum
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub enum TimeInForce {
    GTC, // Good Till Cancelled
    IOC, // Immediate Or Cancel
    FOK, // Fill Or Kill
}

impl TimeInForce {
    pub fn as_str(&self) -> &'static str {
        match self {
            TimeInForce::GTC => "GTC",
            TimeInForce::IOC => "IOC",
            TimeInForce::FOK => "FOK",
        }
    }

    pub fn from_str(s: &str) -> Result<Self, String> {
        match s.to_uppercase().as_str() {
            "GTC" => Ok(TimeInForce::GTC),
            "IOC" => Ok(TimeInForce::IOC),
            "FOK" => Ok(TimeInForce::FOK),
            _ => Err(format!("Unknown time in force: {}", s)),
        }
    }
}

// Market model
#[derive(Debug, Clone, Queryable, Selectable, Serialize, Deserialize)]
#[diesel(table_name = markets)]
pub struct Market {
    pub id: String, // UUID as String
    pub base_asset: String,
    pub quote_asset: String,
    pub default_maker_fee: BigDecimal,
    pub default_taker_fee: BigDecimal,
    pub create_time: i64,
    pub update_time: i64,
    pub status: String,
    pub min_base_amount: BigDecimal,
    pub min_quote_amount: BigDecimal,
    pub price_precision: i32,
    pub amount_precision: i32,
}

impl Market {
    pub fn get_status(&self) -> Result<MarketStatus, String> {
        match self.status.as_str() {
            "ACTIVE" => Ok(MarketStatus::Active),
            "CLOSED" => Ok(MarketStatus::Closed),
            _ => Err(format!("Unknown market status: {}", self.status)),
        }
    }
}

// New Market for insertion
#[derive(Debug, Clone, Insertable, Serialize, Deserialize)]
#[diesel(table_name = markets)]
pub struct NewMarket {
    pub id: String,
    pub base_asset: String,
    pub quote_asset: String,
    pub default_maker_fee: BigDecimal,
    pub default_taker_fee: BigDecimal,
    pub create_time: i64,
    pub update_time: i64,
    pub status: String,
    pub min_base_amount: BigDecimal,
    pub min_quote_amount: BigDecimal,
    pub price_precision: i32,
    pub amount_precision: i32,
}

// Order model
#[derive(Debug, Clone, Queryable, Selectable, Serialize, Deserialize)]
#[diesel(belongs_to(Market))]
#[diesel(table_name = orders)]
pub struct Order {
    pub id: String,
    pub market_id: String,
    pub user_id: String,
    pub order_type: String, // Will be converted to/from OrderType enum
    pub side: String,       // Will be converted to/from OrderSide enum
    pub price: BigDecimal,
    pub base_amount: BigDecimal,
    pub quote_amount: BigDecimal,
    pub maker_fee: BigDecimal,
    pub taker_fee: BigDecimal,
    pub create_time: i64,
    pub remained_base: BigDecimal,
    pub remained_quote: BigDecimal,
    pub filled_base: BigDecimal,
    pub filled_quote: BigDecimal,
    pub filled_fee: BigDecimal,
    pub update_time: i64,
    pub status: String, // Will be converted to/from OrderStatus enum
    pub client_order_id: Option<String>,
    pub post_only: Option<bool>,
    pub time_in_force: Option<String>,
    pub expires_at: Option<i64>,
}

// Helper methods to work with enums
impl Order {
    pub fn get_order_type(&self) -> Result<OrderType, String> {
        OrderType::from_str(&self.order_type)
    }

    pub fn get_side(&self) -> Result<OrderSide, String> {
        OrderSide::from_str(&self.side)
    }

    pub fn get_status(&self) -> Result<OrderStatus, String> {
        OrderStatus::from_str(&self.status)
    }
}

// New Order for insertion
#[derive(Debug, Clone, Insertable, Serialize, Deserialize)]
#[diesel(belongs_to(Market))]
#[diesel(table_name = orders)]
pub struct NewOrder {
    pub id: String,
    pub market_id: String,
    pub user_id: String,
    pub order_type: String,
    pub side: String,
    pub price: BigDecimal,
    pub base_amount: BigDecimal,
    pub quote_amount: BigDecimal,
    pub maker_fee: BigDecimal,
    pub taker_fee: BigDecimal,
    pub create_time: i64,
    pub remained_base: BigDecimal,
    pub remained_quote: BigDecimal,
    pub filled_base: BigDecimal,
    pub filled_quote: BigDecimal,
    pub filled_fee: BigDecimal,
    pub update_time: i64,
    pub status: String,
    pub client_order_id: Option<String>,
    pub post_only: Option<bool>,
    pub time_in_force: Option<String>,
    pub expires_at: Option<i64>,
}

// Trade model
#[derive(Debug, Clone, Queryable, Selectable, Serialize, Deserialize)]
#[diesel(belongs_to(Market))]
#[diesel(table_name = trades)]
pub struct Trade {
    pub id: String,
    pub timestamp: i64,
    pub market_id: String,
    pub price: BigDecimal,
    pub base_amount: BigDecimal,
    pub quote_amount: BigDecimal,
    pub buyer_user_id: String,
    pub buyer_order_id: String,
    pub buyer_fee: BigDecimal,
    pub seller_user_id: String,
    pub seller_order_id: String,
    pub seller_fee: BigDecimal,
    pub taker_side: String,
    pub is_liquidation: Option<bool>,
}

// New Trade for insertion
#[derive(Debug, Clone, Insertable, Serialize, Deserialize)]
#[diesel(belongs_to(Market))]
#[diesel(table_name = trades)]
pub struct NewTrade {
    pub id: String,
    pub timestamp: i64,
    pub market_id: String,
    pub price: BigDecimal,
    pub base_amount: BigDecimal,
    pub quote_amount: BigDecimal,
    pub buyer_user_id: String,
    pub buyer_order_id: String,
    pub buyer_fee: BigDecimal,
    pub seller_user_id: String,
    pub seller_order_id: String,
    pub seller_fee: BigDecimal,
    pub taker_side: String,
    pub is_liquidation: Option<bool>,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub enum MarketStatus {
    Active,
    Closed, // Market is closed and no longer accepting orders
}

impl MarketStatus {
    pub fn as_str(&self) -> &'static str {
        match self {
            MarketStatus::Active => "ACTIVE",
            MarketStatus::Closed => "CLOSED",
        }
    }
}

// Balance model
#[derive(Debug, Clone, Queryable, Selectable, Serialize, Deserialize)]
#[diesel(primary_key(user_id, asset))]
#[diesel(table_name = wallets)]
pub struct Wallet {
    pub user_id: String,
    pub asset: String,
    pub available: BigDecimal,
    pub locked: BigDecimal,
    pub update_time: i64,
    pub reserved: BigDecimal,
    pub total_deposited: BigDecimal,
    pub total_withdrawn: BigDecimal,
}

// New Balance for insertion
#[derive(Debug, Clone, Insertable, Serialize, Deserialize)]
#[diesel(table_name = wallets)]
pub struct NewWallet {
    pub user_id: String,
    pub asset: String,
    pub available: BigDecimal,
    pub locked: BigDecimal,
    pub update_time: i64,
    pub reserved: BigDecimal,
    pub total_deposited: BigDecimal,
    pub total_withdrawn: BigDecimal,
}

// Market Stats model
#[derive(Debug, Clone, Queryable, Selectable, Serialize, Deserialize)]
#[diesel(belongs_to(Market))]
#[diesel(table_name = market_stats)]
pub struct MarketStat {
    pub market_id: String,
    pub high_24h: BigDecimal,
    pub low_24h: BigDecimal,
    pub volume_24h: BigDecimal,
    pub price_change_24h: BigDecimal,
    pub last_price: BigDecimal,
    pub last_update_time: i64,
}

#[derive(Debug, Clone, Insertable, Serialize, Deserialize)]
#[diesel(table_name = market_stats)]
pub struct NewMarketStat {
    pub market_id: String,
    pub high_24h: BigDecimal,
    pub low_24h: BigDecimal,
    pub volume_24h: BigDecimal,
    pub price_change_24h: BigDecimal,
    pub last_price: BigDecimal,
    pub last_update_time: i64,
}
// Fee Treasury model
#[derive(Debug, Clone, Queryable, Selectable, Serialize, Deserialize)]
#[diesel(belongs_to(Market))]
#[diesel(primary_key(market_id, asset))]
#[diesel(table_name = fee_treasury)]
pub struct FeeTreasury {
    pub market_id: String,
    pub asset: String,
    pub treasury_address: String,
    pub collected_amount: BigDecimal,
    pub last_update_time: i64,
}

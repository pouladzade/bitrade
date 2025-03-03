// models.rs
// Diesel ORM models corresponding to database tables

use bigdecimal::BigDecimal;
use diesel::prelude::*;
use serde::{Deserialize, Serialize};

use super::schema::*;

// Represents the OrderType enum in your application
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub enum OrderType {
    Limit,
    Market,
    // Add other order types as needed
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

// Represents the OrderStatus enum
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
            _ => Err(format!("Unknown order status: {}", s)),
        }
    }
}

// Market model
#[derive(Debug, Clone, Queryable, Identifiable, Insertable, Serialize, Deserialize)]
#[diesel(table_name = markets)]
pub struct Market {
    pub id: String,
    pub base_asset: String,
    pub quote_asset: String,
    pub default_maker_fee: BigDecimal,
    pub default_taker_fee: BigDecimal,
    pub create_time: i64,
    pub update_time: i64,
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
}

// Order model
#[derive(Debug, Clone, Queryable, Identifiable, Associations, Serialize, Deserialize)]
#[diesel(belongs_to(Market))]
#[diesel(table_name = orders)]
pub struct Order {
    pub id: String,
    pub market_id: String,
    pub user_id: String,
    pub order_type: String, // Will be converted to/from OrderType enum
    pub side: String,       // Will be converted to/from OrderSide enum
    pub price: BigDecimal,
    pub amount: BigDecimal,
    pub maker_fee: BigDecimal,
    pub taker_fee: BigDecimal,
    pub create_time: i64,
    pub remain: BigDecimal,    
    pub filled_base: BigDecimal,
    pub filled_quote: BigDecimal,
    pub filled_fee: BigDecimal,
    pub update_time: i64,
    pub status: String, // Will be converted to/from OrderStatus enum
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
#[diesel(table_name = orders)]
pub struct NewOrder {
    pub id: String,
    pub market_id: String,
    pub user_id: String,
    pub order_type: String,
    pub side: String,
    pub price: BigDecimal,
    pub amount: BigDecimal,
    pub maker_fee: BigDecimal,
    pub taker_fee: BigDecimal,
    pub create_time: i64,
    pub remain: BigDecimal,
    pub filled_base: BigDecimal,
    pub filled_quote: BigDecimal,
    pub filled_fee: BigDecimal,
    pub update_time: i64,
    pub status: String,
}

// Trade model
#[derive(Debug, Clone, Queryable, Identifiable, Associations, Serialize, Deserialize)]
#[diesel(belongs_to(Market))]
#[diesel(table_name = trades)]
pub struct Trade {
    pub id: String,
    pub timestamp: i64,
    pub market_id: String,
    pub price: BigDecimal,
    pub amount: BigDecimal,
    pub quote_amount: BigDecimal,
    pub taker_user_id: String,
    pub taker_order_id: String,
    pub taker_fee: BigDecimal,
    pub maker_user_id: String,
    pub maker_order_id: String,
    pub maker_fee: BigDecimal,
}

// New Trade for insertion
#[derive(Debug, Clone, Insertable, Serialize, Deserialize)]
#[diesel(table_name = trades)]
pub struct NewTrade {
    pub id: String,
    pub timestamp: i64,
    pub market_id: String,
    pub price: BigDecimal,
    pub amount: BigDecimal,
    pub quote_amount: BigDecimal,
    pub taker_user_id: String,
    pub taker_order_id: String,

    pub taker_fee: BigDecimal,
    pub maker_user_id: String,
    pub maker_order_id: String,
    pub maker_fee: BigDecimal,
}

// Balance model
#[derive(Debug, Clone, Queryable, Identifiable, Serialize, Deserialize)]
#[diesel(primary_key(user_id, asset))]
#[diesel(table_name = balances)]
pub struct Balance {
    pub user_id: String,
    pub asset: String,
    pub available: BigDecimal,
    pub frozen: BigDecimal,
    pub update_time: i64,
}

// New Balance for insertion
#[derive(Debug, Clone, Insertable, Serialize, Deserialize)]
#[diesel(table_name = balances)]
pub struct NewBalance {
    pub user_id: String,
    pub asset: String,
    pub available: BigDecimal,
    pub frozen: BigDecimal,
    pub update_time: i64,
}

// Market Stats model
#[derive(Debug, Clone, Queryable, Identifiable, Associations, Serialize, Deserialize)]
#[diesel(belongs_to(Market))]
#[diesel(primary_key(market_id))]
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

// New Market Stats for insertion
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

-- Your SQL goes here
-- Your SQL goes here
-- Create extensions
CREATE EXTENSION IF NOT EXISTS "uuid-ossp";

-- Markets table
CREATE TABLE markets (
    id VARCHAR(50) PRIMARY KEY,
    base_asset VARCHAR(20) NOT NULL,
    quote_asset VARCHAR(20) NOT NULL,
    default_maker_fee DECIMAL(10, 8) NOT NULL,
    default_taker_fee DECIMAL(10, 8) NOT NULL,
    create_time BIGINT NOT NULL,
    update_time BIGINT NOT NULL,
    
    -- Enforce uniqueness of trading pairs
    UNIQUE (base_asset, quote_asset),
    
    -- Validate fees are non-negative
    CONSTRAINT positive_maker_fee CHECK (default_maker_fee >= 0),
    CONSTRAINT positive_taker_fee CHECK (default_taker_fee >= 0)
);

-- Create index on assets for faster lookups
CREATE INDEX idx_markets_assets ON markets(base_asset, quote_asset);

-- Orders table
CREATE TABLE orders (
    id VARCHAR(50) PRIMARY KEY,
    market_id VARCHAR(50) NOT NULL,
    user_id VARCHAR(50) NOT NULL,
    order_type VARCHAR(20) NOT NULL, -- Limit, Market, etc
    side VARCHAR(10) NOT NULL, -- Buy or Sell
    price DECIMAL(30, 8) NOT NULL,
    amount DECIMAL(30, 8) NOT NULL,
    maker_fee DECIMAL(10, 8) NOT NULL,
    taker_fee DECIMAL(10, 8) NOT NULL,
    create_time BIGINT NOT NULL,
    
    -- Mutable fields
    remain DECIMAL(30, 8) NOT NULL,
    frozen DECIMAL(30, 8) NOT NULL,
    filled_base DECIMAL(30, 8) NOT NULL DEFAULT 0,
    filled_quote DECIMAL(30, 8) NOT NULL DEFAULT 0,
    filled_fee DECIMAL(30, 8) NOT NULL DEFAULT 0,
    update_time BIGINT NOT NULL,
    partially_filled BOOLEAN NOT NULL DEFAULT FALSE,
    
    -- Status field (can be useful)
    status VARCHAR(20) NOT NULL DEFAULT 'OPEN', -- OPEN, FILLED, CANCELED, REJECTED
    
    -- Foreign key to markets
    CONSTRAINT fk_market FOREIGN KEY (market_id) REFERENCES markets(id),
    
    -- Validate that remain and filled_base never exceed amount
    CONSTRAINT valid_amounts CHECK (remain + filled_base <= amount),
    
    -- Validate positive values
    CONSTRAINT positive_price CHECK (price > 0),
    CONSTRAINT positive_amount CHECK (amount > 0),
    CONSTRAINT non_negative_remain CHECK (remain >= 0),
    CONSTRAINT non_negative_frozen CHECK (frozen >= 0),
    CONSTRAINT non_negative_filled_base CHECK (filled_base >= 0),
    CONSTRAINT non_negative_filled_quote CHECK (filled_quote >= 0),
    CONSTRAINT non_negative_filled_fee CHECK (filled_fee >= 0)
);

-- Create composite index for order book queries
CREATE INDEX idx_open_orders ON orders(market_id, side, price) WHERE status = 'OPEN';

-- Create index for user orders
CREATE INDEX idx_user_orders ON orders(user_id, create_time);

-- Trades table
CREATE TABLE trades (
    id VARCHAR(50) PRIMARY KEY,
    timestamp BIGINT NOT NULL,
    market_id VARCHAR(50) NOT NULL,
    price DECIMAL(30, 8) NOT NULL,
    amount DECIMAL(30, 8) NOT NULL,
    quote_amount DECIMAL(30, 8) NOT NULL,
    
    taker_user_id VARCHAR(50) NOT NULL,
    taker_order_id VARCHAR(50) NOT NULL,    
    taker_fee DECIMAL(30, 8) NOT NULL,
    
    maker_user_id VARCHAR(50) NOT NULL,
    maker_order_id VARCHAR(50) NOT NULL,    
    maker_fee DECIMAL(30, 8) NOT NULL,
    
    -- Foreign keys
    CONSTRAINT fk_market_trade FOREIGN KEY (market_id) REFERENCES markets(id),
    CONSTRAINT fk_taker_order FOREIGN KEY (taker_order_id) REFERENCES orders(id),
    CONSTRAINT fk_maker_order FOREIGN KEY (maker_order_id) REFERENCES orders(id),
    
    -- Validate positive values
    CONSTRAINT positive_trade_price CHECK (price > 0),
    CONSTRAINT positive_trade_amount CHECK (amount > 0),
    CONSTRAINT positive_quote_amount CHECK (quote_amount > 0),
    CONSTRAINT non_negative_taker_fee CHECK (taker_fee >= 0),
    CONSTRAINT non_negative_maker_fee CHECK (maker_fee >= 0)
);

-- Create index for market trades (price history)
CREATE INDEX idx_market_trades ON trades(market_id, timestamp);

-- Create index for user trades
CREATE INDEX idx_user_trades ON trades(taker_user_id, maker_user_id, timestamp);

-- Create index for order trades (to quickly find trades for a specific order)
CREATE INDEX idx_order_trades ON trades(taker_order_id, maker_order_id);

-- Optional: User balances table to track user assets
CREATE TABLE balances (
    user_id VARCHAR(50) NOT NULL,
    asset VARCHAR(20) NOT NULL,
    available DECIMAL(30, 8) NOT NULL DEFAULT 0,
    frozen DECIMAL(30, 8) NOT NULL DEFAULT 0,
    update_time BIGINT NOT NULL,
    
    -- Composite primary key
    PRIMARY KEY (user_id, asset),
    
    -- Validate non-negative balances
    CONSTRAINT non_negative_available CHECK (available >= 0),
    CONSTRAINT non_negative_frozen CHECK (frozen >= 0)
);

-- Optional: Create a table for market stats
CREATE TABLE market_stats (
    market_id VARCHAR(50) PRIMARY KEY,
    high_24h DECIMAL(30, 8) NOT NULL DEFAULT 0,
    low_24h DECIMAL(30, 8) NOT NULL DEFAULT 0,
    volume_24h DECIMAL(30, 8) NOT NULL DEFAULT 0,
    price_change_24h DECIMAL(30, 8) NOT NULL DEFAULT 0,
    last_price DECIMAL(30, 8) NOT NULL DEFAULT 0,
    last_update_time BIGINT NOT NULL,
    
    CONSTRAINT fk_market_stats FOREIGN KEY (market_id) REFERENCES markets(id)
);
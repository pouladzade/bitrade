-- Create extensions
CREATE EXTENSION IF NOT EXISTS "uuid-ossp";

-- Markets table
CREATE TABLE markets (
    id VARCHAR(36) PRIMARY KEY,
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
    CONSTRAINT positive_taker_fee CHECK (default_taker_fee >= 0),
    
    -- New columns
    status VARCHAR(20) NOT NULL DEFAULT 'ACTIVE',
    min_base_amount DECIMAL(30, 8) NOT NULL DEFAULT 0,
    min_quote_amount DECIMAL(30, 8) NOT NULL DEFAULT 0,
    price_precision INT NOT NULL DEFAULT 8,
    amount_precision INT NOT NULL DEFAULT 8,
    
    -- Enhanced constraints
    CONSTRAINT valid_precision CHECK (price_precision BETWEEN 0 AND 18),
    CONSTRAINT valid_amount_precision CHECK (amount_precision BETWEEN 0 AND 18),
    CONSTRAINT valid_status CHECK (status IN ('ACTIVE', 'INACTIVE', 'SUSPENDED')),
    CONSTRAINT valid_min_amounts CHECK (min_base_amount >= 0 AND min_quote_amount >= 0)
);

-- Create index on assets for faster lookups
CREATE INDEX idx_markets_assets ON markets(base_asset, quote_asset);

-- Orders table
CREATE TABLE orders (
    id VARCHAR(36) PRIMARY KEY,
    market_id VARCHAR(36) NOT NULL,
    user_id VARCHAR(36) NOT NULL,
    order_type VARCHAR(20) NOT NULL, -- Limit, Market, etc
    side VARCHAR(10) NOT NULL, -- Buy or Sell
    price DECIMAL(30, 8) NOT NULL,
    base_amount DECIMAL(30, 8) NOT NULL,
    quote_amount DECIMAL(30, 8) NOT NULL,
    maker_fee DECIMAL(10, 8) NOT NULL,
    taker_fee DECIMAL(10, 8) NOT NULL,
    create_time BIGINT NOT NULL,
    
    -- Mutable fields
    remained_base DECIMAL(30, 8) NOT NULL,
    remained_quote DECIMAL(30, 8) NOT NULL,
    
    filled_base DECIMAL(30, 8) NOT NULL DEFAULT 0,
    filled_quote DECIMAL(30, 8) NOT NULL DEFAULT 0,
    filled_fee DECIMAL(30, 8) NOT NULL DEFAULT 0,
    update_time BIGINT NOT NULL,
    
    

    status VARCHAR(20) NOT NULL DEFAULT 'OPEN', -- OPEN, FILLED, CANCELED, REJECTED, PARTIALLY_FILLED
    
    -- Foreign key to markets
    CONSTRAINT fk_market FOREIGN KEY (market_id) REFERENCES markets(id),
    
    -- Validate that remain and filled_base never exceed amount

    CONSTRAINT valid_amounts CHECK (remained_base + filled_base <= base_amount),
    
    -- Validate positive values

    CONSTRAINT positive_base_amount CHECK (base_amount > 0),
    CONSTRAINT non_negative_quote_amount CHECK (quote_amount >= 0),
    CONSTRAINT non_negative_remained_base CHECK (remained_base >= 0),
    CONSTRAINT non_negative_remained_quote CHECK (remained_quote >= 0),
    CONSTRAINT non_negative_price CHECK (price >= 0),
    CONSTRAINT non_negative_filled_base CHECK (filled_base >= 0),
    CONSTRAINT non_negative_filled_quote CHECK (filled_quote >= 0),
    CONSTRAINT non_negative_filled_fee CHECK (filled_fee >= 0),
    
    -- New columns
    client_order_id VARCHAR(50),
    post_only BOOLEAN DEFAULT FALSE,
    time_in_force VARCHAR(10) DEFAULT 'GTC', -- GTC, IOC, FOK
    expires_at BIGINT DEFAULT NULL,
    
    -- Enhanced constraints
    CONSTRAINT valid_order_type CHECK (order_type IN ('LIMIT', 'MARKET', 'STOP_LIMIT', 'STOP_MARKET')),
    CONSTRAINT valid_side CHECK (side IN ('BUY', 'SELL')),
    CONSTRAINT valid_order_status CHECK (status IN ('OPEN', 'FILLED', 'CANCELED', 'REJECTED', 'PARTIALLY_FILLED')),
    CONSTRAINT valid_time_in_force CHECK (time_in_force IN ('GTC', 'IOC', 'FOK')),
    CONSTRAINT valid_expires_at CHECK (
        (time_in_force = 'GTC' AND expires_at IS NULL) OR 
        (time_in_force IN ('IOC', 'FOK') AND expires_at IS NOT NULL)
    )
);

-- Create composite index for order book queries
CREATE INDEX idx_open_orders ON orders(market_id, side, price) WHERE status = 'OPEN';

-- Create index for user orders
CREATE INDEX idx_user_orders ON orders(user_id, create_time);

-- Create unique index for user client order id
CREATE UNIQUE INDEX idx_user_client_order_id ON orders(user_id, client_order_id) WHERE client_order_id IS NOT NULL;

-- Trades table with buyer/seller orientation
CREATE TABLE trades (
    id VARCHAR(36) PRIMARY KEY,
    timestamp BIGINT NOT NULL,
    market_id VARCHAR(36) NOT NULL,
    price DECIMAL(30, 8) NOT NULL,
    base_amount DECIMAL(30, 8) NOT NULL,
    quote_amount DECIMAL(30, 8) NOT NULL,
    
    buyer_user_id VARCHAR(36) NOT NULL,
    buyer_order_id VARCHAR(36) NOT NULL,    
    buyer_fee DECIMAL(30, 8) NOT NULL,
    
    seller_user_id VARCHAR(36) NOT NULL,
    seller_order_id VARCHAR(36) NOT NULL,    
    seller_fee DECIMAL(30, 8) NOT NULL,
    
    taker_side VARCHAR(10) NOT NULL, -- 'BUYER' or 'SELLER' - indicates which side was the taker
    is_liquidation BOOLEAN DEFAULT FALSE,
    
    -- Foreign keys
    CONSTRAINT fk_market_trade FOREIGN KEY (market_id) REFERENCES markets(id),
    CONSTRAINT fk_buyer_order FOREIGN KEY (buyer_order_id) REFERENCES orders(id),
    CONSTRAINT fk_seller_order FOREIGN KEY (seller_order_id) REFERENCES orders(id),
    
    -- Validate positive values
    CONSTRAINT positive_trade_price CHECK (price > 0),
    CONSTRAINT positive_trade_base_amount CHECK (base_amount > 0),
    CONSTRAINT positive_trade_quote_amount CHECK (quote_amount > 0),
    CONSTRAINT non_negative_buyer_fee CHECK (buyer_fee >= 0),
    CONSTRAINT non_negative_seller_fee CHECK (seller_fee >= 0),
    
    -- Enhanced constraints
    CONSTRAINT valid_taker_side CHECK (taker_side IN ('BUY', 'SELL')),
    
    CONSTRAINT different_users CHECK (buyer_user_id != seller_user_id)
);

-- Create index for market trades (price history)
CREATE INDEX idx_market_trades ON trades(market_id, timestamp);

-- Create indices for user trades - one for each role
CREATE INDEX idx_buyer_trades ON trades(buyer_user_id, timestamp);
CREATE INDEX idx_seller_trades ON trades(seller_user_id, timestamp);

-- Create indices for order trades
CREATE INDEX idx_buyer_order_trades ON trades(buyer_order_id);
CREATE INDEX idx_seller_order_trades ON trades(seller_order_id);

-- Optional: User balances table to track user assets
CREATE TABLE wallets (
    user_id VARCHAR(36) NOT NULL,
    asset VARCHAR(20) NOT NULL,
    available DECIMAL(30, 8) NOT NULL DEFAULT 0,
    locked DECIMAL(30, 8) NOT NULL DEFAULT 0,
    update_time BIGINT NOT NULL,
    
    -- Composite primary key
    PRIMARY KEY (user_id, asset),
    
    -- Validate non-negative balances
    CONSTRAINT non_negative_available CHECK (available >= 0),
    CONSTRAINT non_negative_locked CHECK (locked >= 0),
    
    -- New columns
    reserved DECIMAL(30, 8) NOT NULL DEFAULT 0,
    total_deposited DECIMAL(30, 8) NOT NULL DEFAULT 0,
    total_withdrawn DECIMAL(30, 8) NOT NULL DEFAULT 0,
    
    -- Enhanced constraints
    CONSTRAINT non_negative_reserved CHECK (reserved >= 0),
    CONSTRAINT non_negative_deposited CHECK (total_deposited >= 0),
    CONSTRAINT non_negative_withdrawn CHECK (total_withdrawn >= 0),
    CONSTRAINT valid_total CHECK (available + locked + reserved >= 0)
);

-- Optional: Create a table for market stats
CREATE TABLE market_stats (
    market_id VARCHAR(36) PRIMARY KEY,
    high_24h DECIMAL(30, 8) NOT NULL DEFAULT 0,
    low_24h DECIMAL(30, 8) NOT NULL DEFAULT 0,
    volume_24h DECIMAL(30, 8) NOT NULL DEFAULT 0,
    price_change_24h DECIMAL(30, 8) NOT NULL DEFAULT 0,
    last_price DECIMAL(30, 8) NOT NULL DEFAULT 0,
    last_update_time BIGINT NOT NULL,
    
    CONSTRAINT fk_market_stats FOREIGN KEY (market_id) REFERENCES markets(id)
);

-- Fee Treasury table with composite primary key
CREATE TABLE fee_treasury (
    market_id VARCHAR(36) NOT NULL,
    asset VARCHAR(20) NOT NULL,
    treasury_address VARCHAR(100) NOT NULL,
    collected_amount DECIMAL(30, 8) NOT NULL DEFAULT 0,
    last_update_time BIGINT NOT NULL,
    
    -- Make market_id and asset the primary key
    PRIMARY KEY (market_id, asset),
    
    -- Foreign key to markets
    CONSTRAINT fk_market_treasury FOREIGN KEY (market_id) REFERENCES markets(id),
    
    -- Validate non-negative balance
    CONSTRAINT non_negative_collected_fees CHECK (collected_amount >= 0)
);

-- Index for treasury_address queries if needed
CREATE INDEX idx_fee_treasury_address ON fee_treasury(treasury_address);

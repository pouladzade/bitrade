-- This file should undo anything in `up.sql`
-- This file should undo anything in `up.sql`
-- Drop the market_stats table
DROP TABLE IF EXISTS market_stats;

-- Drop the balances table
DROP TABLE IF EXISTS balances;

-- Drop the indexes on the trades table
DROP INDEX IF EXISTS idx_order_trades;
DROP INDEX IF EXISTS idx_user_trades;
DROP INDEX IF EXISTS idx_market_trades;

-- Drop the trades table
DROP TABLE IF EXISTS trades;

-- Drop the indexes on the orders table
DROP INDEX IF EXISTS idx_user_orders;
DROP INDEX IF EXISTS idx_open_orders;

-- Drop the orders table
DROP TABLE IF EXISTS orders;

-- Drop the index on the markets table
DROP INDEX IF EXISTS idx_markets_assets;

-- Drop the markets table
DROP TABLE IF EXISTS markets;

-- Drop the uuid-ossp extension
DROP EXTENSION IF EXISTS "uuid-ossp";
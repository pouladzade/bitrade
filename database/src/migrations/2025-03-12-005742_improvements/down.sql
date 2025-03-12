-- Drop indices
DROP INDEX IF EXISTS idx_fee_treasury_market_asset;
DROP INDEX IF EXISTS idx_seller_order_trades;
DROP INDEX IF EXISTS idx_buyer_order_trades;
DROP INDEX IF EXISTS idx_seller_trades;
DROP INDEX IF EXISTS idx_buyer_trades;
DROP INDEX IF EXISTS idx_market_trades;
DROP INDEX IF EXISTS idx_user_client_order_id;
DROP INDEX IF EXISTS idx_user_orders;
DROP INDEX IF EXISTS idx_open_orders;

-- Drop tables in reverse order of creation
DROP TABLE IF EXISTS trades;
DROP TABLE IF EXISTS Wallets;
DROP TABLE IF EXISTS market_stats;
DROP TABLE IF EXISTS fee_treasury;
DROP TABLE IF EXISTS orders;
DROP TABLE IF EXISTS markets;

-- Drop extensions
DROP EXTENSION IF EXISTS "uuid-ossp";
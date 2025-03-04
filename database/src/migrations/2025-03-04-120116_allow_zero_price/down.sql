-- This file should undo anything in `up.sql`
ALTER TABLE orders DROP CONSTRAINT non_negative_price;
ALTER TABLE orders ADD CONSTRAINT positive_price CHECK (price > 0);

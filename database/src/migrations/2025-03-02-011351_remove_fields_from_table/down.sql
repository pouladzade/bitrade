-- This file should undo anything in `up.sql`
ALTER TABLE orders ADD COLUMN frozen DECIMAL(30, 8) NOT NULL;
ALTER TABLE orders ADD partially_filled BOOLEAN NOT NULL DEFAULT FALSE
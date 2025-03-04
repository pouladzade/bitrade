-- Your SQL goes here
ALTER TABLE orders DROP CONSTRAINT positive_price;
ALTER TABLE orders ADD CONSTRAINT non_negative_price CHECK (price >= 0);

-- Add migration script here
ALTER TABLE courses
ADD COLUMN paypal_product_id TEXT;

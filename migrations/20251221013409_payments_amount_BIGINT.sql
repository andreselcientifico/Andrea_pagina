-- Add migration script here
ALTER TABLE payments
ADD COLUMN amount_tmp BIGINT NOT NULL DEFAULT 0;
UPDATE payments
SET amount_tmp = CAST(amount * 100 AS BIGINT);
ALTER TABLE payments
DROP COLUMN amount;
ALTER TABLE payments
RENAME COLUMN amount_tmp TO amount;
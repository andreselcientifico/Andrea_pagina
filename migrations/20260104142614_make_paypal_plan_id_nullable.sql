-- Make paypal_plan_id nullable in subscription_plans table
ALTER TABLE subscription_plans ALTER COLUMN paypal_plan_id DROP NOT NULL;

-- Add migration script here
ALTER TABLE subscription_plans
ADD COLUMN trial_days INTEGER DEFAULT NULL;

ALTER TABLE subscription_plans
ADD CONSTRAINT trial_days_positive
CHECK (trial_days IS NULL OR trial_days > 0);

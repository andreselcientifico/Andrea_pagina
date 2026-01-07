-- Add migration script here
-- Alterar tabla achievement para agregar campos faltantes
ALTER TABLE achievement ADD COLUMN IF NOT EXISTS trigger_type VARCHAR(50) NOT NULL DEFAULT 'manual';
ALTER TABLE achievement ADD COLUMN IF NOT EXISTS trigger_value INTEGER NOT NULL DEFAULT 1;
ALTER TABLE achievement ADD COLUMN IF NOT EXISTS active BOOLEAN NOT NULL DEFAULT TRUE;

-- Alterar tabla subscription_plans para agregar features
ALTER TABLE subscription_plans ADD COLUMN IF NOT EXISTS features JSONB DEFAULT '[]'::jsonb;
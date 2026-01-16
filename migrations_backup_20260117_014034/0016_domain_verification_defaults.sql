-- Migration 0016: Domain Verification Defaults
-- Adds DEFAULT value to verification_token column for automatic generation

ALTER TABLE auth_tenants 
ALTER COLUMN verification_token SET DEFAULT encode(gen_random_bytes(24), 'base64');

-- Ensure all existing tenants have tokens (if any were missed)
UPDATE auth_tenants SET verification_token = encode(gen_random_bytes(24), 'base64') WHERE verification_token IS NULL;

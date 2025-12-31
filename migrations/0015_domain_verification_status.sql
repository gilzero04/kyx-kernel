-- Migration 0015: Domain Verification Status
-- Adds support for Cloudflare-style custom domain verification
CREATE EXTENSION IF NOT EXISTS "pgcrypto";

ALTER TABLE auth_tenants 
ADD COLUMN domain_verified_at TIMESTAMP WITH TIME ZONE,
ADD COLUMN verification_token VARCHAR(64) UNIQUE;

-- Generate initial tokens for existing tenants
UPDATE auth_tenants SET verification_token = encode(gen_random_bytes(24), 'base64') WHERE verification_token IS NULL;

COMMENT ON COLUMN auth_tenants.domain_verified_at IS 'Timestamp of last successful DNS verification';
COMMENT ON COLUMN auth_tenants.verification_token IS 'Unique token for TXT record verification';

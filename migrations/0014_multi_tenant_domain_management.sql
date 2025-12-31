-- ================================================
-- Migration: 0014_multi_tenant_domain_management.sql
-- Purpose: Add Custom Domain and Subdomain support (WordPress Multisite-style)
-- ================================================

-- 1. Add Domain-related columns to auth_tenants
ALTER TABLE auth_tenants 
ADD COLUMN IF NOT EXISTS custom_domain VARCHAR(255) UNIQUE,
ADD COLUMN IF NOT EXISTS allow_child_subdomains BOOLEAN DEFAULT FALSE,
ADD COLUMN IF NOT EXISTS use_parent_subdomain BOOLEAN DEFAULT FALSE;

-- 2. Index for fast lookup by domain
CREATE INDEX IF NOT EXISTS idx_auth_tenants_custom_domain ON auth_tenants(custom_domain) WHERE custom_domain IS NOT NULL;

-- 3. Comments for clarity
COMMENT ON COLUMN auth_tenants.custom_domain IS 'Optional custom domain for white-labeling (e.g., www.client.com)';
COMMENT ON COLUMN auth_tenants.allow_child_subdomains IS 'If true, children of this tenant can use subdomains under this tenant''s domain';
COMMENT ON COLUMN auth_tenants.use_parent_subdomain IS 'If true, this tenant can be accessed as child.parent.domain.com (only if parent allows)';

-- ================================================
-- Migration: 20251228140000_init_landing_config.sql
-- Purpose: Implement Dynamic Landing Page and Tenant Type defaults
-- ================================================

-- 1. Add landing_path to sys_roles (Runtime Value)
-- This stores the actual path a user with this role should be redirected to.
ALTER TABLE sys_roles ADD COLUMN IF NOT EXISTS landing_path VARCHAR(255);

-- 2. Create Defaults Configuration Table (Template)
-- This defines the "Standard Rule": "If Tenant Type is X and Role is Y, then Landing is Z"
CREATE TABLE IF NOT EXISTS sys_tenant_type_role_defaults (
    tenant_type_id UUID NOT NULL REFERENCES sys_tenant_types(id) ON DELETE CASCADE,
    role_slug VARCHAR(50) NOT NULL,
    default_landing_path VARCHAR(255) NOT NULL,
    created_at TIMESTAMPTZ DEFAULT NOW(),
    PRIMARY KEY (tenant_type_id, role_slug)
);

-- 3. Seed Expanded Tenant Types (The "Super App" Vision)
INSERT INTO sys_tenant_types (slug, name, description, icon, is_system) VALUES 
    ('platform', 'Platform Owner', 'System Owner and Administrator', 'server', TRUE),
    ('marketplace', 'Marketplace', 'Aggregator of vendors', 'shopping-bag', TRUE),
    ('hospital', 'Hospital', 'Healthcare provider', 'activity', TRUE),
    ('logistics', 'Logistics', 'Transport and warehouse provider', 'truck', TRUE),
    ('service', 'Service Provider', 'General service provider', 'tool', TRUE)
ON CONFLICT (slug) DO NOTHING;

-- 4. Seed Default Landing Logic
-- Helper function to get type ID
CREATE OR REPLACE FUNCTION get_tenant_type_id(slug_in VARCHAR) RETURNS UUID AS $$
    SELECT id FROM sys_tenant_types WHERE slug = slug_in;
$$ LANGUAGE SQL;

INSERT INTO sys_tenant_type_role_defaults (tenant_type_id, role_slug, default_landing_path) VALUES
    -- Platform (Owner)
    (get_tenant_type_id('platform'), 'superadmin', '/admin/dashboard'),
    (get_tenant_type_id('platform'), 'viewer', '/admin/overview'),
    
    -- Marketplace (Standard Client)
    (get_tenant_type_id('marketplace'), 'superadmin', '/marketplace/dashboard'),
    (get_tenant_type_id('marketplace'), 'vendor', '/marketplace/vendor-portal'),
    
    -- Vendor (Sub-client)
    (get_tenant_type_id('vendor'), 'superadmin', '/vendor/dashboard'),
    (get_tenant_type_id('vendor'), 'staff', '/vendor/orders'),

    -- Hospital
    (get_tenant_type_id('hospital'), 'superadmin', '/hospital/admin'),
    (get_tenant_type_id('hospital'), 'doctor', '/hospital/patients'),

    -- Standard (Fallback)
    (get_tenant_type_id('standard'), 'superadmin', '/app/dashboard'),
    (get_tenant_type_id('standard'), 'viewer', '/app/dashboard')
ON CONFLICT (tenant_type_id, role_slug) DO UPDATE
SET default_landing_path = EXCLUDED.default_landing_path;

-- Cleanup helper
DROP FUNCTION get_tenant_type_id(VARCHAR);

-- 5. Backfill existing roles (Optional but good for dev)
-- Set Platform SuperAdmin default
UPDATE sys_roles 
SET landing_path = '/admin/dashboard' 
WHERE slug = 'superadmin' AND tenant_id IN (SELECT id FROM auth_tenants WHERE parent_id IS NULL);

-- Set Standard SuperAdmin default
UPDATE sys_roles 
SET landing_path = '/app/dashboard' 
WHERE slug = 'superadmin' AND tenant_id IN (SELECT id FROM auth_tenants WHERE parent_id IS NOT NULL);

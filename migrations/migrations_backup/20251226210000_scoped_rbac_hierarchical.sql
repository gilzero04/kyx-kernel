-- ================================================
-- Migration: 20251226210000_scoped_rbac_hierarchical.sql
-- Purpose: Enable scoped roles and hierarchical permission delegation
-- ================================================

-- 1. Update sys_roles uniqueness to be tenant-scoped
-- This allows different tenants to have a role with the same slug (e.g., 'superadmin')
ALTER TABLE sys_roles DROP CONSTRAINT IF EXISTS sys_roles_slug_key;
CREATE UNIQUE INDEX IF NOT EXISTS idx_sys_roles_scoped_slug ON sys_roles (tenant_id, slug) WHERE tenant_id IS NOT NULL;
CREATE UNIQUE INDEX IF NOT EXISTS idx_sys_roles_global_slug ON sys_roles (slug) WHERE tenant_id IS NULL;

-- 2. Enhance permissions with system-level flag
ALTER TABLE sys_permissions ADD COLUMN IF NOT EXISTS is_system BOOLEAN DEFAULT FALSE;

-- Mark existing system-critical permissions
UPDATE sys_permissions SET is_system = TRUE WHERE slug IN ('system:manage', 'plugin:install');

-- 3. Create Tenant Permission Delegation Table
-- This table defines which permissions are "available" to be used by a tenant
CREATE TABLE IF NOT EXISTS sys_tenant_permissions (
    tenant_id UUID NOT NULL REFERENCES auth_tenants(id) ON DELETE CASCADE,
    permission_id UUID NOT NULL REFERENCES sys_permissions(id) ON DELETE CASCADE,
    created_at TIMESTAMPTZ DEFAULT NOW(),
    PRIMARY KEY (tenant_id, permission_id)
);

-- 4. Set initial scope for System Owner
-- The master organization (parent_id IS NULL) gets access to ALL permissions
INSERT INTO sys_tenant_permissions (tenant_id, permission_id)
SELECT t.id, p.id 
FROM auth_tenants t, sys_permissions p
WHERE t.parent_id IS NULL
ON CONFLICT DO NOTHING;

-- 5. Set initial scope for existing Child Tenants
-- Existing child tenants get all NON-system permissions by default
INSERT INTO sys_tenant_permissions (tenant_id, permission_id)
SELECT t.id, p.id 
FROM auth_tenants t, sys_permissions p
WHERE t.parent_id IS NOT NULL AND p.is_system = FALSE
ON CONFLICT DO NOTHING;

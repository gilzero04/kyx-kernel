-- Migration: 20251226220000_granular_permissions.sql
-- Purpose: Seed granular action-based permissions and update default roles.

-- 1. Insert Granular Permissions
INSERT INTO sys_permissions (code, slug, name, description, is_system) VALUES 
    -- User Module
    ('U.R', 'user:read', 'Read Users', 'Can view user lists and profiles', FALSE),
    ('U.C', 'user:create', 'Create Users', 'Can add new users to the organization', FALSE),
    ('U.U', 'user:update', 'Update Users', 'Can modify user details and status', FALSE),
    ('U.D', 'user:delete', 'Delete Users', 'Can deactivate users', FALSE),
    
    -- Tenant Module
    ('T.R', 'tenant:read', 'Read Tenants', 'Can view tenant hierarchy and details', FALSE),
    ('T.C', 'tenant:create', 'Create Tenants', 'Can create sub-organizations', FALSE),
    ('T.U', 'tenant:update', 'Update Tenants', 'Can modify organization settings', FALSE),
    ('T.D', 'tenant:delete', 'Delete Tenants', 'Can deactivate organizations', FALSE),
    
    -- RBAC Module
    ('R.R', 'rbac:read', 'Read RBAC', 'Can view roles and permissions', FALSE),
    ('R.C', 'rbac:create', 'Create Roles', 'Can create new access roles', FALSE),
    ('R.U', 'rbac:update', 'Update Roles', 'Can modify role names and mappings', FALSE),
    ('R.D', 'rbac:delete', 'Delete Roles', 'Can deactivate roles', FALSE),
    
    -- Audit Module
    ('A.R', 'audit:read', 'Read Audit Logs', 'Can view activity history', FALSE),
    
    -- System Actions (Root Only)
    ('S.M', 'system:manage', 'System Management', 'Full access to core kernel settings', TRUE)
ON CONFLICT (slug) DO UPDATE SET 
    name = EXCLUDED.name,
    description = EXCLUDED.description,
    is_system = EXCLUDED.is_system;

-- 2. Update Role Permissions
-- We want to map these to standard roles for the System Owner first.
-- For child tenants, the delegation transaction in PostgresTenantRepository handles it.

-- Viewer Role Mapping
INSERT INTO sys_role_permissions (role_id, permission_id)
SELECT r.id, p.id FROM sys_roles r, sys_permissions p
WHERE r.slug = 'viewer' AND p.slug IN ('user:read', 'tenant:read', 'rbac:read', 'audit:read')
ON CONFLICT DO NOTHING;

-- Operator Role Mapping
INSERT INTO sys_role_permissions (role_id, permission_id)
SELECT r.id, p.id FROM sys_roles r, sys_permissions p
WHERE r.slug = 'operator' AND p.slug IN ('user:read', 'user:create', 'user:update', 'tenant:read', 'rbac:read', 'audit:read')
ON CONFLICT DO NOTHING;

-- SuperAdministrator Mapping (Full Access)
INSERT INTO sys_role_permissions (role_id, permission_id)
SELECT r.id, p.id FROM sys_roles r, sys_permissions p
WHERE r.slug = 'superadmin'
ON CONFLICT DO NOTHING;

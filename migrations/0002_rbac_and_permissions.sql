-- ================================================
-- Migration: 0002_rbac_and_permissions.sql
-- Purpose: Hierarchical RBAC scoping and granular action-based permissions.
-- ================================================

-- 1. Support Scoped Permission Delegation (System vs. Tenant)
-- Allows the System Owner to delegate specific permissions to tenants.
CREATE TABLE IF NOT EXISTS sys_tenant_permissions (
    tenant_id UUID NOT NULL REFERENCES auth_tenants(id) ON DELETE CASCADE,
    permission_id UUID NOT NULL REFERENCES sys_permissions(id) ON DELETE CASCADE,
    PRIMARY KEY (tenant_id, permission_id)
);

-- Seed Initial System Permissions
INSERT INTO sys_permissions (code, slug, name, description, is_system) VALUES 
    -- SYSTEM (Infrastructure)
    ('S.M', 'system:manage', 'Full System Management', 'Unrestricted administrative access', TRUE),
    ('S.C.R', 'system:config:read', 'Read Configuration', 'Can view kernel and module settings', TRUE),
    ('S.C.U', 'system:config:update', 'Update Configuration', 'Can modify kernel and module settings', TRUE),
    ('S.A.M', 'system:api_key:manage', 'Manage API Keys', 'Can create and revoke system API keys', TRUE),
    ('S.O.M', 'system:cors:manage', 'Manage CORS', 'Can manage cross-origin resource sharing policies', TRUE),
    ('S.L.R', 'system:audit:read', 'Read Audit Logs', 'Can view system-wide activity history', TRUE),
    ('S.I.M', 'system:i18n:manage', 'Manage I18n', 'Can update translations and locales', TRUE),
    
    -- ROLE (RBAC)
    ('R.R', 'role:read', 'Read Roles', 'Can view access roles', FALSE),
    ('R.C', 'role:create', 'Create Roles', 'Can create new access roles', FALSE),
    ('R.U', 'role:update', 'Update Roles', 'Can modify role names and permission mappings', FALSE),
    ('R.D', 'role:delete', 'Delete Roles', 'Can deactivate roles', FALSE),
    
    -- PERMISSION (RBAC)
    ('P.R', 'permission:read', 'Read Permissions', 'Can view permission list', FALSE),
    ('P.C', 'permission:create', 'Create Permissions', 'Can create new permission definitions', FALSE),
    ('P.U', 'permission:update', 'Update Permissions', 'Can modify permission details', FALSE),
    ('P.D', 'permission:delete', 'Delete Permissions', 'Can remove permission definitions', FALSE),

    -- USER (Identity)
    ('U.R', 'user:read', 'Read Users', 'Can view user list and profiles', FALSE),
    ('U.C', 'user:create', 'Create Users', 'Can add new users', FALSE),
    ('U.U', 'user:update', 'Update Users', 'Can modify user details and status', FALSE),
    ('U.D', 'user:delete', 'Delete Users', 'Can deactivate or remove users', FALSE),
    ('U.P', 'user:reset_password', 'Reset Passwords', 'Can trigger password resets for users', FALSE),
    
    -- TENANT (Organization)
    ('T.R', 'tenant:read', 'Read Tenants', 'Can view organization forest and details', FALSE),
    ('T.C', 'tenant:create', 'Create Tenants', 'Can create sub-organizations', FALSE),
    ('T.U', 'tenant:update', 'Update Tenants', 'Can modify organization settings', FALSE),
    ('T.D', 'tenant:delete', 'Delete Tenants', 'Can deactivate organizations', FALSE)
ON CONFLICT (slug) DO UPDATE SET 
    code = EXCLUDED.code,
    name = EXCLUDED.name,
    description = EXCLUDED.description,
    is_system = EXCLUDED.is_system;

-- 2. Initial System Roles (Seed if not exists)
INSERT INTO sys_roles (slug, name, description, sort_order) VALUES 
    ('superadmin', 'Super Administrator', 'Full access within the scoped organization', 100),
    ('admin', 'Administrator', 'Administrative access', 80),
    ('operator', 'Operator', 'Operation access', 60),
    ('viewer', 'Viewer', 'Read-only access', 40)
ON CONFLICT (tenant_id, slug) WHERE tenant_id IS NULL DO NOTHING;

-- 3. Global SuperAdmin Mapping (Unscoped)
INSERT INTO sys_role_permissions (role_id, permission_id)
SELECT r.id, p.id FROM sys_roles r, sys_permissions p
WHERE r.slug = 'superadmin' AND r.tenant_id IS NULL
ON CONFLICT DO NOTHING;

-- 4. Viewer/Operator Mapping (Unscoped)
INSERT INTO sys_role_permissions (role_id, permission_id)
SELECT r.id, p.id FROM sys_roles r, sys_permissions p
WHERE r.slug = 'viewer' AND r.tenant_id IS NULL AND p.slug IN (
    'user:read', 'tenant:read', 'role:read', 'permission:read', 'system:audit:read', 'system:config:read'
)
ON CONFLICT DO NOTHING;

INSERT INTO sys_role_permissions (role_id, permission_id)
SELECT r.id, p.id FROM sys_roles r, sys_permissions p
WHERE r.slug = 'operator' AND r.tenant_id IS NULL AND p.slug IN (
    'user:read', 'user:create', 'user:update', 'user:reset_password',
    'tenant:read', 'role:read', 'permission:read', 'system:audit:read'
)
ON CONFLICT DO NOTHING;

-- Migration: 20251228120000_comprehensive_permissions_seed.sql
-- Purpose: Complete categorization (SYSTEM, ROLE, PERMISSION, USER, TENANT) and robust seeding.

-- 1. Clear legacy mappings that might conflict with new granular keys (Optional/Safety)
-- We use ON CONFLICT slugs mostly, but for role mapping we might want a fresh start if renaming.

-- 2. Insert Comprehensive Permissions
-- Clean up any existing permissions that might have conflicting slugs or codes to ensure a clean seed
DELETE FROM sys_permissions WHERE slug IN (
    'system:manage', 'system:config:read', 'system:config:update', 'system:api_key:manage', 'system:cors:manage', 'system:audit:read', 'system:i18n:manage',
    'role:read', 'role:create', 'role:update', 'role:delete',
    'permission:read', 'permission:create', 'permission:update', 'permission:delete',
    'user:read', 'user:create', 'user:update', 'user:delete', 'user:reset_password',
    'tenant:read', 'tenant:create', 'tenant:update', 'tenant:delete'
) OR code IN (
    'S.M', 'S.C.R', 'S.C.U', 'S.A.M', 'S.O.M', 'S.L.R', 'S.I.M',
    'R.R', 'R.C', 'R.U', 'R.D',
    'P.R', 'P.C', 'P.U', 'P.D',
    'U.R', 'U.C', 'U.U', 'U.D', 'U.P',
    'T.R', 'T.C', 'T.U', 'T.D'
);

INSERT INTO sys_permissions (code, slug, name, description, is_system) VALUES 
    -- SYSTEM (Admin/Infrastructure)
    ('S.M', 'system:manage', 'Full System Management', 'Unrestricted administrative access', TRUE),
    ('S.C.R', 'system:config:read', 'Read Configuration', 'Can view kernel and module settings', TRUE),
    ('S.C.U', 'system:config:update', 'Update Configuration', 'Can modify kernel and module settings', TRUE),
    ('S.A.M', 'system:api_key:manage', 'Manage API Keys', 'Can create and revoke system API keys', TRUE),
    ('S.O.M', 'system:cors:manage', 'Manage CORS', 'Can manage cross-origin resource sharing policies', TRUE),
    ('S.L.R', 'system:audit:read', 'Read Audit Logs', 'Can view system-wide activity history', TRUE),
    ('S.I.M', 'system:i18n:manage', 'Manage I18n', 'Can update translations and locales', TRUE),
    
    -- ROLE (RBAC - Roles)
    ('R.R', 'role:read', 'Read Roles', 'Can view access roles', FALSE),
    ('R.C', 'role:create', 'Create Roles', 'Can create new access roles', FALSE),
    ('R.U', 'role:update', 'Update Roles', 'Can modify role names and permission mappings', FALSE),
    ('R.D', 'role:delete', 'Delete Roles', 'Can deactivate roles', FALSE),
    
    -- PERMISSION (RBAC - Permissions)
    ('P.R', 'permission:read', 'Read Permissions', 'Can view permission list', FALSE),
    ('P.C', 'permission:create', 'Create Permissions', 'Can create new permission definitions', FALSE),
    ('P.U', 'permission:update', 'Update Permissions', 'Can modify permission details', FALSE),
    ('P.D', 'permission:delete', 'Delete Permissions', 'Can remove permission definitions', FALSE),

    -- USER (Identity Management)
    ('U.R', 'user:read', 'Read Users', 'Can view user list and profiles', FALSE),
    ('U.C', 'user:create', 'Create Users', 'Can add new users', FALSE),
    ('U.U', 'user:update', 'Update Users', 'Can modify user details and status', FALSE),
    ('U.D', 'user:delete', 'Delete Users', 'Can deactivate or remove users', FALSE),
    ('U.P', 'user:reset_password', 'Reset Passwords', 'Can trigger password resets for users', FALSE),
    
    -- TENANT (Organization Management)
    ('T.R', 'tenant:read', 'Read Tenants', 'Can view organization forest and details', FALSE),
    ('T.C', 'tenant:create', 'Create Tenants', 'Can create sub-organizations', FALSE),
    ('T.U', 'tenant:update', 'Update Tenants', 'Can modify organization settings', FALSE),
    ('T.D', 'tenant:delete', 'Delete Tenants', 'Can deactivate organizations', FALSE)
ON CONFLICT (code) DO UPDATE SET 
    slug = EXCLUDED.slug,
    name = EXCLUDED.name,
    description = EXCLUDED.description,
    is_system = EXCLUDED.is_system;

-- 3. Cleanup old/clashing permissions (if rbac:read was used for both roles/perms)
DELETE FROM sys_permissions WHERE slug IN ('rbac:read', 'rbac:create', 'rbac:update', 'rbac:delete');

-- 4. Map everything to SuperAdmin
-- Note: sys_role_permissions has a unique constraint on (role_id, permission_id)
INSERT INTO sys_role_permissions (role_id, permission_id)
SELECT r.id, p.id FROM sys_roles r, sys_permissions p
WHERE r.slug = 'superadmin'
ON CONFLICT DO NOTHING;

-- 5. Map Viewer subsets
INSERT INTO sys_role_permissions (role_id, permission_id)
SELECT r.id, p.id FROM sys_roles r, sys_permissions p
WHERE r.slug = 'viewer' AND p.slug IN (
    'user:read', 'tenant:read', 'role:read', 'permission:read', 'system:audit:read', 'system:config:read'
)
ON CONFLICT DO NOTHING;

-- 6. Map Operator subsets
INSERT INTO sys_role_permissions (role_id, permission_id)
SELECT r.id, p.id FROM sys_roles r, sys_permissions p
WHERE r.slug = 'operator' AND p.slug IN (
    'user:read', 'user:create', 'user:update', 'user:reset_password',
    'tenant:read', 'role:read', 'permission:read', 'system:audit:read'
)
ON CONFLICT DO NOTHING;

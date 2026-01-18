-- ════════════════════════════════════════════════════════════════════════════
-- Migration: 0002_rbac.sql
-- Purpose: RBAC permissions, roles, and role-permission mappings
-- Consolidated from: 0002, 0008, 0012, 0013, 0021, 0022
-- ════════════════════════════════════════════════════════════════════════════

-- ════════════════════════════════════════════════════════════════════════════
-- 1. SYSTEM PERMISSIONS (All granular permissions in one place)
-- ════════════════════════════════════════════════════════════════════════════

INSERT INTO sys_permissions (code, slug, name, description, is_system, sort_order) VALUES 
    -- System (Infrastructure)
    ('S.M', 'system:manage', 'Full System Management', 'Unrestricted administrative access', TRUE, 1),
    ('S.R', 'system:read', 'Read System Info', 'Can view basic system context and status', TRUE, 2),
    ('S.C.R', 'system:config:read', 'Read Configuration', 'Can view kernel and module settings', TRUE, 3),
    ('S.C.U', 'system:config:update', 'Update Configuration', 'Can modify kernel and module settings', TRUE, 4),
    ('S.A.M', 'system:api_key:manage', 'Manage API Keys', 'Can create and revoke system API keys', TRUE, 5),
    ('S.O.M', 'system:cors:manage', 'Manage CORS', 'Can manage cross-origin resource sharing policies', TRUE, 6),
    ('S.L.R', 'system:audit:read', 'Read Audit Logs', 'Can view system-wide activity history', TRUE, 7),
    ('S.I.M', 'system:i18n:manage', 'Manage I18n', 'Can update translations and locales', TRUE, 8),
    
    -- Role (RBAC)
    ('R.R', 'role:read', 'Read Roles', 'Can view access roles', FALSE, 10),
    ('R.C', 'role:create', 'Create Roles', 'Can create new access roles', FALSE, 11),
    ('R.U', 'role:update', 'Update Roles', 'Can modify role names and permission mappings', FALSE, 12),
    ('R.D', 'role:delete', 'Delete Roles', 'Can deactivate roles', FALSE, 13),
    
    -- Permission (RBAC)
    ('P.R', 'permission:read', 'Read Permissions', 'Can view permission list', FALSE, 20),
    ('P.C', 'permission:create', 'Create Permissions', 'Can create new permission definitions', FALSE, 21),
    ('P.U', 'permission:update', 'Update Permissions', 'Can modify permission details', FALSE, 22),
    ('P.D', 'permission:delete', 'Delete Permissions', 'Can remove permission definitions', FALSE, 23),

    -- User (Identity)
    ('U.R', 'user:read', 'Read Users', 'Can view user list and profiles', FALSE, 30),
    ('U.C', 'user:create', 'Create Users', 'Can add new users', FALSE, 31),
    ('U.U', 'user:update', 'Update Users', 'Can modify user details and status', FALSE, 32),
    ('U.D', 'user:delete', 'Delete Users', 'Can deactivate or remove users', FALSE, 33),
    ('U.P', 'user:reset_password', 'Reset Passwords', 'Can trigger password resets for users', FALSE, 34),
    
    -- Tenant (Organization)
    ('T.R', 'tenant:read', 'Read Tenants', 'Can view organization forest and details', FALSE, 40),
    ('T.C', 'tenant:create', 'Create Tenants', 'Can create sub-organizations', FALSE, 41),
    ('T.U', 'tenant:update', 'Update Tenants', 'Can modify organization settings', FALSE, 42),
    ('T.D', 'tenant:delete', 'Delete Tenants', 'Can deactivate organizations', FALSE, 43),
    
    -- Media
    ('M.R', 'media:read', 'Read Assets', 'Can view and download media files', FALSE, 50),
    ('M.U', 'media:upload', 'Upload Assets', 'Can upload new media files', FALSE, 51),
    ('M.D', 'media:delete', 'Delete Assets', 'Can remove media files', FALSE, 52),
    ('M.F', 'media:folder', 'Manage Folders', 'Can create and manage media folders', FALSE, 53),
    
    -- CMS
    ('CMS.R', 'cms:read', 'Read CMS Pages', 'Can view CMS page list and content', FALSE, 60),
    ('CMS.W', 'cms:write', 'Write CMS Pages', 'Can create and edit CMS pages', FALSE, 61),
    ('CMS.D', 'cms:delete', 'Delete CMS Pages', 'Can remove CMS pages', FALSE, 62),
    ('CMS.P', 'cms:publish', 'Publish CMS Pages', 'Can change publication status of CMS pages', FALSE, 63),
    
    -- Plugin (Granular)
    ('PLG.R', 'plugin:read', 'View Plugins', 'Can view installed plugins', TRUE, 200),
    ('PLG.I', 'plugin:install', 'Install Plugins', 'Can install new plugins', TRUE, 201),
    ('PLG.E', 'plugin:enable', 'Enable Plugins', 'Can enable/disable plugins', TRUE, 202),
    ('PLG.U', 'plugin:uninstall', 'Uninstall Plugins', 'Can remove plugins', TRUE, 203),
    ('PLG.C', 'plugin:configure', 'Configure Plugins', 'Can modify plugin settings', TRUE, 204),
    ('PLG.A', 'plugin:approve', 'Approve Plugins', 'Can approve plugins with dangerous capabilities', TRUE, 205),
    
    -- Theme (Granular)
    ('TH.R', 'theme:read', 'Read Themes', 'Allow reading theme data', FALSE, 210),
    ('TH.I', 'theme:import', 'Import Themes', 'Allow importing new themes', FALSE, 211),
    ('TH.A', 'theme:activate', 'Activate Themes', 'Allow activating themes', FALSE, 212),
    ('TH.U', 'theme:update', 'Update Themes', 'Allow updating theme settings', FALSE, 213),
    ('TH.D', 'theme:delete', 'Delete Themes', 'Allow deleting themes', FALSE, 214),
    
    -- i18n (Granular)
    ('I18.R', 'i18n:read', 'Read i18n', 'Allow reading locales and translations', FALSE, 220),
    ('I18.M', 'i18n:manage', 'Manage i18n', 'Allow managing locales, keys, and translations', FALSE, 221)
ON CONFLICT (slug) DO UPDATE SET 
    code = EXCLUDED.code,
    name = EXCLUDED.name,
    description = EXCLUDED.description,
    is_system = EXCLUDED.is_system,
    sort_order = EXCLUDED.sort_order;

-- ════════════════════════════════════════════════════════════════════════════
-- 2. SYSTEM ROLES (Global template roles with NULL tenant_id)
-- ════════════════════════════════════════════════════════════════════════════

-- Initial System Roles (Global templates, tenant_id = NULL)
INSERT INTO sys_roles (slug, name, description, sort_order) VALUES 
    ('superadmin', 'Super Administrator', 'Full access within the scoped organization', 100),
    ('admin', 'Administrator', 'Administrative access', 80),
    ('operator', 'Operator', 'Operation access', 60),
    ('viewer', 'Viewer', 'Read-only access', 40)
ON CONFLICT (tenant_id, slug) WHERE tenant_id IS NULL DO NOTHING;

-- Global SuperAdmin Mapping (All permissions)
INSERT INTO sys_role_permissions (role_id, permission_id)
SELECT r.id, p.id FROM sys_roles r, sys_permissions p
WHERE r.slug = 'superadmin' AND r.tenant_id IS NULL
ON CONFLICT DO NOTHING;

-- Viewer/Operator Mapping (Read-only subset)
INSERT INTO sys_role_permissions (role_id, permission_id)
SELECT r.id, p.id FROM sys_roles r, sys_permissions p
WHERE r.slug = 'viewer' AND r.tenant_id IS NULL AND p.slug IN (
    'user:read', 'tenant:read', 'role:read', 'permission:read', 'system:audit:read', 'system:config:read', 'system:read'
)
ON CONFLICT DO NOTHING;

INSERT INTO sys_role_permissions (role_id, permission_id)
SELECT r.id, p.id FROM sys_roles r, sys_permissions p
WHERE r.slug = 'operator' AND r.tenant_id IS NULL AND p.slug IN (
    'user:read', 'user:create', 'user:update', 'user:reset_password',
    'tenant:read', 'role:read', 'permission:read', 'system:audit:read', 'system:read'
)
ON CONFLICT DO NOTHING;

-- ════════════════════════════════════════════════════════════════════════════
-- COMMENTS
-- ════════════════════════════════════════════════════════════════════════════

COMMENT ON TABLE sys_permissions IS 'Granular permissions for RBAC';
COMMENT ON TABLE sys_roles IS 'Roles scoped to tenants (NULL tenant_id = global template)';
COMMENT ON TABLE sys_role_permissions IS 'Role to permission mappings';

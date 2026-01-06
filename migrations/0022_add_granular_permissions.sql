-- ================================================
-- Migration: 0022_add_granular_permissions.sql
-- Purpose: Add granular permissions for plugins, themes, and i18n
-- ================================================

-- Plugin granular permissions (existing: plugin:read)
INSERT INTO sys_permissions (id, name, slug, description, created_at, updated_at) VALUES
    (gen_random_uuid(), 'Install Plugins', 'plugin:install', 'Allow installing new plugins', NOW(), NOW()),
    (gen_random_uuid(), 'Enable/Disable Plugins', 'plugin:enable', 'Allow enabling or disabling plugins', NOW(), NOW()),
    (gen_random_uuid(), 'Uninstall Plugins', 'plugin:uninstall', 'Allow uninstalling plugins', NOW(), NOW()),
    (gen_random_uuid(), 'Configure Plugins', 'plugin:configure', 'Allow updating plugin configuration', NOW(), NOW()),
    (gen_random_uuid(), 'Approve Plugins', 'plugin:approve', 'Allow approving plugins with security risks', NOW(), NOW())
ON CONFLICT (slug) DO NOTHING;

-- Theme granular permissions (replace legacy theme:write)
INSERT INTO sys_permissions (id, name, slug, description, created_at, updated_at) VALUES
    (gen_random_uuid(), 'Read Themes', 'theme:read', 'Allow reading theme data', NOW(), NOW()),
    (gen_random_uuid(), 'Import Themes', 'theme:import', 'Allow importing new themes', NOW(), NOW()),
    (gen_random_uuid(), 'Activate Themes', 'theme:activate', 'Allow activating themes', NOW(), NOW()),
    (gen_random_uuid(), 'Update Themes', 'theme:update', 'Allow updating theme settings', NOW(), NOW()),
    (gen_random_uuid(), 'Delete Themes', 'theme:delete', 'Allow deleting themes', NOW(), NOW())
ON CONFLICT (slug) DO NOTHING;

-- i18n granular permissions (replace legacy system:i18n:manage)
INSERT INTO sys_permissions (id, name, slug, description, created_at, updated_at) VALUES
    (gen_random_uuid(), 'Read i18n', 'i18n:read', 'Allow reading locales and translations', NOW(), NOW()),
    (gen_random_uuid(), 'Manage i18n', 'i18n:manage', 'Allow managing locales, keys, and translations', NOW(), NOW())
ON CONFLICT (slug) DO NOTHING;

-- Assign all new permissions to superadmin role
INSERT INTO sys_role_permissions (role_id, permission_id)
SELECT r.id, p.id 
FROM sys_roles r
CROSS JOIN sys_permissions p
WHERE r.slug = 'superadmin' 
  AND r.tenant_id IS NULL
  AND p.slug IN (
    'plugin:install', 'plugin:enable', 'plugin:uninstall', 'plugin:configure', 'plugin:approve',
    'theme:read', 'theme:import', 'theme:activate', 'theme:update', 'theme:delete',
    'i18n:read', 'i18n:manage'
  )
ON CONFLICT DO NOTHING;


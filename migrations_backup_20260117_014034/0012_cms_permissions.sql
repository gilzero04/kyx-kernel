-- ================================================
-- Migration: 0012_cms_permissions.sql
-- Purpose: Add CMS management permissions and map them to SuperAdmin.
-- ================================================

-- 1. Create CMS Permissions
INSERT INTO sys_permissions (code, slug, name, description, is_system) VALUES 
    ('CMS.R', 'cms:read', 'Read CMS Pages', 'Can view CMS page list and content', FALSE),
    ('CMS.W', 'cms:write', 'Write CMS Pages', 'Can create and edit CMS pages', FALSE),
    ('CMS.D', 'cms:delete', 'Delete CMS Pages', 'Can remove CMS pages', FALSE),
    ('CMS.P', 'cms:publish', 'Publish CMS Pages', 'Can change publication status of CMS pages', FALSE)
ON CONFLICT (slug) DO UPDATE SET 
    code = EXCLUDED.code,
    name = EXCLUDED.name,
    description = EXCLUDED.description,
    is_system = EXCLUDED.is_system;

-- 2. Assign to Global SuperAdmin (Unscoped)
INSERT INTO sys_role_permissions (role_id, permission_id)
SELECT r.id, p.id FROM sys_roles r, sys_permissions p
WHERE r.slug = 'superadmin' AND r.tenant_id IS NULL
AND p.slug IN ('cms:read', 'cms:write', 'cms:delete', 'cms:publish')
ON CONFLICT DO NOTHING;

-- 3. Assign to existing System Owner Tenant and its SuperAdmin role
DO $$
DECLARE
    owner_id UUID;
BEGIN
    -- Find the system owner (root tenant)
    SELECT id INTO owner_id FROM auth_tenants WHERE parent_id = id AND deleted_at IS NULL LIMIT 1;
    
    IF owner_id IS NOT NULL THEN
        -- Add to tenant permissions (Owner gets all)
        INSERT INTO sys_tenant_permissions (tenant_id, permission_id)
        SELECT owner_id, id FROM sys_permissions 
        WHERE slug IN ('cms:read', 'cms:write', 'cms:delete', 'cms:publish')
        ON CONFLICT DO NOTHING;
        
        -- Add to owner's superadmin role specifically
        INSERT INTO sys_role_permissions (role_id, permission_id)
        SELECT r.id, p.id FROM sys_roles r, sys_permissions p
        WHERE r.tenant_id = owner_id AND r.slug = 'superadmin'
        AND p.slug IN ('cms:read', 'cms:write', 'cms:delete', 'cms:publish')
        ON CONFLICT DO NOTHING;
    END IF;
END $$;

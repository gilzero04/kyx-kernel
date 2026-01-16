-- ================================================
-- Migration: 0008_media_permissions.sql
-- Purpose: Add media management permissions and map them to SuperAdmin.
-- ================================================

-- 1. Create Media Permissions
INSERT INTO sys_permissions (code, slug, name, description, is_system) VALUES 
    ('M.R', 'media:read', 'Read Assets', 'Can view and download media files', FALSE),
    ('M.U', 'media:upload', 'Upload Assets', 'Can upload new media files', FALSE),
    ('M.D', 'media:delete', 'Delete Assets', 'Can remove media files', FALSE),
    ('M.F', 'media:folder', 'Manage Folders', 'Can create and manage media folders', FALSE)
ON CONFLICT (slug) DO UPDATE SET 
    code = EXCLUDED.code,
    name = EXCLUDED.name,
    description = EXCLUDED.description,
    is_system = EXCLUDED.is_system;

-- 2. Assign to Global SuperAdmin (Unscoped)
INSERT INTO sys_role_permissions (role_id, permission_id)
SELECT r.id, p.id FROM sys_roles r, sys_permissions p
WHERE r.slug = 'superadmin' AND r.tenant_id IS NULL
AND p.slug IN ('media:read', 'media:upload', 'media:delete', 'media:folder')
ON CONFLICT DO NOTHING;

-- 3. Assign to existing System Owner Tenant and its SuperAdmin role
DO $$
DECLARE
    owner_id UUID;
BEGIN
    -- Find the system owner (root tenant)
    SELECT id INTO owner_id FROM auth_tenants WHERE parent_id IS NULL AND deleted_at IS NULL LIMIT 1;
    
    IF owner_id IS NOT NULL THEN
        -- Add to tenant permissions (Owner gets all)
        INSERT INTO sys_tenant_permissions (tenant_id, permission_id)
        SELECT owner_id, id FROM sys_permissions 
        WHERE slug IN ('media:read', 'media:upload', 'media:delete', 'media:folder')
        ON CONFLICT DO NOTHING;
        
        -- Add to owner's superadmin role specifically
        INSERT INTO sys_role_permissions (role_id, permission_id)
        SELECT r.id, p.id FROM sys_roles r, sys_permissions p
        WHERE r.tenant_id = owner_id AND r.slug = 'superadmin'
        AND p.slug IN ('media:read', 'media:upload', 'media:delete', 'media:folder')
        ON CONFLICT DO NOTHING;
    END IF;
END $$;

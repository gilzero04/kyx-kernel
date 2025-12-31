-- ================================================
-- Migration: 0013_system_read_permission.sql
-- Purpose: Add system:read permission for context/status checks.
-- ================================================

-- 1. Create system:read Permission
INSERT INTO sys_permissions (code, slug, name, description, is_system) VALUES 
    ('S.R', 'system:read', 'Read System Info', 'Can view basic system context and status', TRUE)
ON CONFLICT (slug) DO UPDATE SET 
    code = EXCLUDED.code,
    name = EXCLUDED.name,
    description = EXCLUDED.description,
    is_system = EXCLUDED.is_system;

-- 2. Assign to Roles (Unscoped)
INSERT INTO sys_role_permissions (role_id, permission_id)
SELECT r.id, p.id FROM sys_roles r, sys_permissions p
WHERE p.slug = 'system:read'
AND r.slug IN ('superadmin', 'admin', 'operator', 'viewer')
AND r.tenant_id IS NULL
ON CONFLICT DO NOTHING;

-- 3. Assign to Owner Tenant and its Roles
DO $$
DECLARE
    owner_id UUID;
BEGIN
    SELECT id INTO owner_id FROM auth_tenants WHERE parent_id = id AND deleted_at IS NULL LIMIT 1;
    
    IF owner_id IS NOT NULL THEN
        -- Link permission to tenant
        INSERT INTO sys_tenant_permissions (tenant_id, permission_id)
        SELECT owner_id, id FROM sys_permissions WHERE slug = 'system:read'
        ON CONFLICT DO NOTHING;
        
        -- Link to owner's roles
        INSERT INTO sys_role_permissions (role_id, permission_id)
        SELECT r.id, p.id FROM sys_roles r, sys_permissions p
        WHERE r.tenant_id = owner_id 
        AND r.slug IN ('superadmin', 'admin', 'operator', 'viewer')
        AND p.slug = 'system:read'
        ON CONFLICT DO NOTHING;
    END IF;
END $$;

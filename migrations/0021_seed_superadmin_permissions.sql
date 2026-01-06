-- ================================================
-- Migration: 0021_seed_superadmin_permissions.sql
-- Purpose: Associate all permissions with the superadmin role
-- ================================================

-- Ensure all permissions are assigned to the global superadmin role
INSERT INTO sys_role_permissions (role_id, permission_id)
SELECT r.id, p.id 
FROM sys_roles r
CROSS JOIN sys_permissions p
WHERE r.slug = 'superadmin' AND r.tenant_id IS NULL
ON CONFLICT DO NOTHING;

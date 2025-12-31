-- ================================================
-- Migration: 00000000000003_init_rbac.sql
-- Purpose: Create RBAC tables (roles, permissions, mappings)
-- Depends on: auth_tenants
-- ================================================

-- =====================
-- 1. Permissions
-- =====================
CREATE TABLE IF NOT EXISTS sys_permissions (
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    code VARCHAR(50) UNIQUE,
    slug VARCHAR(100) UNIQUE NOT NULL,
    name VARCHAR(255) NOT NULL,
    description TEXT,
    sort_order INT DEFAULT 0,
    is_active BOOLEAN DEFAULT TRUE,
    created_at TIMESTAMPTZ DEFAULT NOW(),
    updated_at TIMESTAMPTZ DEFAULT NOW(),
    deleted_at TIMESTAMPTZ
);

DROP TRIGGER IF EXISTS update_sys_permissions_updated_at ON sys_permissions;
CREATE TRIGGER update_sys_permissions_updated_at
BEFORE UPDATE ON sys_permissions
FOR EACH ROW EXECUTE FUNCTION update_updated_at_column();

-- Seed default permissions
INSERT INTO sys_permissions (code, slug, name, description) VALUES 
    ('P001', 'system:manage', 'System Management', 'Full system control'),
    ('P002', 'user:write', 'User Management', 'Create/Edit users'),
    ('P003', 'plugin:install', 'Plugin Management', 'Install/Update plugins')
ON CONFLICT (slug) DO NOTHING;

-- =====================
-- 2. Roles
-- =====================
CREATE TABLE IF NOT EXISTS sys_roles (
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    tenant_id UUID REFERENCES auth_tenants(id) ON DELETE CASCADE,
    code VARCHAR(50) UNIQUE,
    name VARCHAR(255) NOT NULL,
    slug VARCHAR(100) UNIQUE NOT NULL,
    description TEXT,
    sort_order INT DEFAULT 0,
    is_active BOOLEAN DEFAULT TRUE,
    max_members INT,
    created_at TIMESTAMPTZ DEFAULT NOW(),
    updated_at TIMESTAMPTZ DEFAULT NOW(),
    deleted_at TIMESTAMPTZ
);

DROP TRIGGER IF EXISTS update_sys_roles_updated_at ON sys_roles;
CREATE TRIGGER update_sys_roles_updated_at
BEFORE UPDATE ON sys_roles
FOR EACH ROW EXECUTE FUNCTION update_updated_at_column();

-- Seed default roles
INSERT INTO sys_roles (code, slug, name, description, sort_order, max_members) VALUES 
    ('R001', 'superadmin', 'Super Administrator', 'Full system access', 100, 3),
    ('R002', 'admin', 'Administrator', 'Administrative access', 80, NULL),
    ('R003', 'operator', 'Operator', 'System operation access', 60, NULL),
    ('R004', 'viewer', 'Viewer', 'Read-only access', 40, NULL)
ON CONFLICT (slug) DO UPDATE SET 
    description = EXCLUDED.description,
    max_members = EXCLUDED.max_members;

-- =====================
-- 3. Tenant Role Limits (Per-tenant overrides)
-- =====================
CREATE TABLE IF NOT EXISTS sys_tenant_role_limits (
    tenant_id UUID REFERENCES auth_tenants(id) ON DELETE CASCADE,
    role_id UUID REFERENCES sys_roles(id) ON DELETE CASCADE,
    max_members INT NOT NULL,
    created_at TIMESTAMPTZ DEFAULT NOW(),
    updated_at TIMESTAMPTZ DEFAULT NOW(),
    PRIMARY KEY (tenant_id, role_id)
);

-- =====================
-- 4. Role-Permission Mapping
-- =====================
CREATE TABLE IF NOT EXISTS sys_role_permissions (
    role_id UUID REFERENCES sys_roles(id) ON DELETE CASCADE,
    permission_id UUID REFERENCES sys_permissions(id) ON DELETE CASCADE,
    PRIMARY KEY (role_id, permission_id)
);

-- Map SuperAdmin to all permissions
INSERT INTO sys_role_permissions (role_id, permission_id)
SELECT r.id, p.id FROM sys_roles r, sys_permissions p
WHERE r.slug = 'superadmin'
ON CONFLICT DO NOTHING;

-- ================================================
-- Migration: 00000000000002_init_auth_core.sql
-- Purpose: Create auth core tables (users, tenant types, tenants)
-- ================================================

-- =====================
-- 1. Users
-- =====================
CREATE TABLE IF NOT EXISTS auth_users (
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    email VARCHAR(255) UNIQUE NOT NULL,
    hashed_password VARCHAR(255) NOT NULL,
    full_name VARCHAR(255),
    is_active BOOLEAN DEFAULT TRUE,
    created_at TIMESTAMPTZ DEFAULT NOW(),
    updated_at TIMESTAMPTZ DEFAULT NOW(),
    deleted_at TIMESTAMPTZ
);

DROP TRIGGER IF EXISTS update_auth_users_updated_at ON auth_users;
CREATE TRIGGER update_auth_users_updated_at
BEFORE UPDATE ON auth_users
FOR EACH ROW EXECUTE FUNCTION update_updated_at_column();

-- =====================
-- 2. Tenant Types (Lookup)
-- =====================
CREATE TABLE IF NOT EXISTS sys_tenant_types (
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    slug VARCHAR(50) UNIQUE NOT NULL,
    name VARCHAR(255) NOT NULL,
    description TEXT,
    icon VARCHAR(50),
    is_system BOOLEAN DEFAULT FALSE,
    is_active BOOLEAN DEFAULT TRUE,
    created_at TIMESTAMPTZ DEFAULT NOW(),
    updated_at TIMESTAMPTZ DEFAULT NOW(),
    deleted_at TIMESTAMPTZ
);

DROP TRIGGER IF EXISTS update_sys_tenant_types_updated_at ON sys_tenant_types;
CREATE TRIGGER update_sys_tenant_types_updated_at
BEFORE UPDATE ON sys_tenant_types
FOR EACH ROW EXECUTE FUNCTION update_updated_at_column();

-- Seed default tenant types
INSERT INTO sys_tenant_types (slug, name, description, icon, is_system) VALUES 
    ('standard', 'Standard', 'Default tenant type', 'building', TRUE),
    ('branch', 'Branch', 'Branch office or location', 'store', TRUE),
    ('vendor', 'Vendor', 'Vendor or seller in marketplace', 'shop', TRUE),
    ('supplier', 'Supplier', 'Supplier or provider', 'truck', TRUE)
ON CONFLICT (slug) DO NOTHING;

-- =====================
-- 3. Tenants (Hierarchical)
-- =====================
CREATE TABLE IF NOT EXISTS auth_tenants (
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    name VARCHAR(255) NOT NULL,
    slug VARCHAR(255) UNIQUE NOT NULL,
    parent_id UUID REFERENCES auth_tenants(id) ON DELETE SET NULL,
    tenant_type_id UUID REFERENCES sys_tenant_types(id),
    is_active BOOLEAN DEFAULT TRUE,
    created_at TIMESTAMPTZ DEFAULT NOW(),
    updated_at TIMESTAMPTZ DEFAULT NOW(),
    deleted_at TIMESTAMPTZ
);

CREATE INDEX IF NOT EXISTS idx_tenants_parent_id ON auth_tenants(parent_id);
CREATE INDEX IF NOT EXISTS idx_tenants_type_id ON auth_tenants(tenant_type_id);

DROP TRIGGER IF EXISTS update_auth_tenants_updated_at ON auth_tenants;
CREATE TRIGGER update_auth_tenants_updated_at
BEFORE UPDATE ON auth_tenants
FOR EACH ROW EXECUTE FUNCTION update_updated_at_column();

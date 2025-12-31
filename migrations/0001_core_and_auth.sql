-- ================================================
-- Migration: 0001_core_and_auth.sql
-- Purpose: Core system functions, system tables, hierarchical tenants, and memberships.
-- ================================================

-- 1. Global Functions
CREATE OR REPLACE FUNCTION update_updated_at_column()
RETURNS TRIGGER AS $$
BEGIN
    NEW.updated_at = NOW();
    RETURN NEW;
END;
$$ language 'plpgsql';

-- 2. System Foundation
CREATE TABLE IF NOT EXISTS audit_logs (
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    timestamp TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    actor TEXT NOT NULL,
    action TEXT NOT NULL,
    target TEXT,
    status TEXT NOT NULL,
    metadata JSONB,
    created_at TIMESTAMPTZ DEFAULT NOW()
);

CREATE INDEX idx_audit_logs_actor ON audit_logs(actor);

CREATE TABLE IF NOT EXISTS sys_configs (
    key VARCHAR(255) PRIMARY KEY,
    value JSONB NOT NULL,
    description TEXT,
    is_public BOOLEAN DEFAULT FALSE,
    tenant_id UUID,
    created_at TIMESTAMPTZ DEFAULT NOW(),
    updated_at TIMESTAMPTZ DEFAULT NOW()
);

CREATE TRIGGER update_sys_configs_updated_at BEFORE UPDATE ON sys_configs FOR EACH ROW EXECUTE FUNCTION update_updated_at_column();

CREATE TABLE IF NOT EXISTS sys_api_keys (
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    tenant_id UUID,
    key_hash VARCHAR(255) NOT NULL,
    prefix VARCHAR(10) NOT NULL,
    name VARCHAR(255),
    key_type VARCHAR(50) DEFAULT 'server',
    allowed_origins JSONB,
    is_active BOOLEAN DEFAULT TRUE,
    created_at TIMESTAMPTZ DEFAULT NOW(),
    updated_at TIMESTAMPTZ DEFAULT NOW(),
    deleted_at TIMESTAMPTZ
);

CREATE TRIGGER update_sys_api_keys_updated_at BEFORE UPDATE ON sys_api_keys FOR EACH ROW EXECUTE FUNCTION update_updated_at_column();

CREATE TABLE IF NOT EXISTS sys_cors_origins (
    id SERIAL PRIMARY KEY,
    origin VARCHAR(255) UNIQUE NOT NULL,
    is_active BOOLEAN DEFAULT TRUE,
    description TEXT,
    created_at TIMESTAMPTZ DEFAULT NOW(),
    updated_at TIMESTAMPTZ DEFAULT NOW(),
    deleted_at TIMESTAMPTZ
);

CREATE TRIGGER update_sys_cors_origins_updated_at BEFORE UPDATE ON sys_cors_origins FOR EACH ROW EXECUTE FUNCTION update_updated_at_column();

INSERT INTO sys_cors_origins (origin, description) VALUES 
    ('http://localhost:5173', 'Vite Default Port'),
    ('http://localhost:5175', 'Local Platform Development'),
    ('http://localhost:4173', 'Vite Preview Port')
ON CONFLICT (origin) DO NOTHING;

-- 3. Auth Core: Users
CREATE TABLE IF NOT EXISTS auth_users (
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    email VARCHAR(255) UNIQUE NOT NULL,
    hashed_password VARCHAR(255) NOT NULL,
    full_name VARCHAR(255),
    avatar_url TEXT,
    cover_url TEXT,
    is_active BOOLEAN DEFAULT TRUE,
    created_at TIMESTAMPTZ DEFAULT NOW(),
    updated_at TIMESTAMPTZ DEFAULT NOW(),
    deleted_at TIMESTAMPTZ
);

CREATE TRIGGER update_auth_users_updated_at BEFORE UPDATE ON auth_users FOR EACH ROW EXECUTE FUNCTION update_updated_at_column();

-- 4. Tenant Types
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

CREATE TRIGGER update_sys_tenant_types_updated_at BEFORE UPDATE ON sys_tenant_types FOR EACH ROW EXECUTE FUNCTION update_updated_at_column();

-- Seed comprehensive tenant types
INSERT INTO sys_tenant_types (slug, name, description, icon, is_system) VALUES 
    ('owner', 'System Owner / เจ้าของระบบ', 'Core platform administration', 'shield-check', TRUE),
    ('branch', 'Branch / สาขา', 'Organization branch or division', 'store', TRUE),
    ('vendor', 'Vendor / ผู้ขาย', 'Marketplace seller or merchant', 'shop', TRUE),
    ('supplier', 'Supplier / ผู้จัดจำหน่าย', 'Inventory or goods provider', 'truck', TRUE),
    ('partner', 'Partner / พันธมิตร', 'External business partner', 'handshake', TRUE),
    ('franchise', 'Franchise / แฟรนไชส์', 'Franchise business model', 'award', TRUE),
    ('service_provider', 'Service Provider / ผู้ให้บริการ', 'Third-party service provider', 'briefcase', TRUE),
    ('marketplace', 'Marketplace', 'Aggregator of vendors', 'shopping-bag', TRUE),
    ('platform', 'Platform Owner', 'Multi-tenant platform administrator', 'server', TRUE)
ON CONFLICT (slug) DO NOTHING;

-- 5. Tenants (Hierarchical with Profile and Branding)
CREATE TABLE IF NOT EXISTS auth_tenants (
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    parent_id UUID REFERENCES auth_tenants(id) ON DELETE SET NULL,
    tenant_type_id UUID REFERENCES sys_tenant_types(id),
    name VARCHAR(255) NOT NULL,
    slug VARCHAR(255) UNIQUE NOT NULL,
    
    -- Branding & White Labeling
    logo_url TEXT,
    logo_dark_url TEXT,
    favicon_url TEXT,
    icon_app_url TEXT,
    primary_color VARCHAR(50),
    secondary_color VARCHAR(50),
    accent_color VARCHAR(50),
    app_name_override VARCHAR(255),
    
    -- Profile & Contact
    contact_email VARCHAR(255),
    contact_phone VARCHAR(50),
    website_url TEXT,
    social_links JSONB DEFAULT '{}'::jsonb,
    address TEXT,
    business_type VARCHAR(100),
    
    -- Config & System
    config JSONB DEFAULT '{}'::jsonb,
    is_active BOOLEAN DEFAULT TRUE,
    created_at TIMESTAMPTZ DEFAULT NOW(),
    updated_at TIMESTAMPTZ DEFAULT NOW(),
    deleted_at TIMESTAMPTZ
);

CREATE INDEX idx_tenants_parent_id ON auth_tenants(parent_id);
CREATE TRIGGER update_auth_tenants_updated_at BEFORE UPDATE ON auth_tenants FOR EACH ROW EXECUTE FUNCTION update_updated_at_column();

-- 6. RBAC Base: Roles & Permissions
CREATE TABLE IF NOT EXISTS sys_permissions (
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    code VARCHAR(50) UNIQUE,
    slug VARCHAR(100) UNIQUE NOT NULL,
    name VARCHAR(255) NOT NULL,
    description TEXT,
    sort_order INT DEFAULT 0,
    is_active BOOLEAN DEFAULT TRUE,
    is_system BOOLEAN DEFAULT FALSE,
    created_at TIMESTAMPTZ DEFAULT NOW(),
    updated_at TIMESTAMPTZ DEFAULT NOW(),
    deleted_at TIMESTAMPTZ
);

CREATE TRIGGER update_sys_permissions_updated_at BEFORE UPDATE ON sys_permissions FOR EACH ROW EXECUTE FUNCTION update_updated_at_column();

CREATE TABLE IF NOT EXISTS sys_roles (
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    tenant_id UUID REFERENCES auth_tenants(id) ON DELETE CASCADE,
    code VARCHAR(50),
    name VARCHAR(255) NOT NULL,
    slug VARCHAR(100) NOT NULL,
    description TEXT,
    landing_path VARCHAR(255),
    sort_order INT DEFAULT 0,
    is_active BOOLEAN DEFAULT TRUE,
    max_members INT,
    created_at TIMESTAMPTZ DEFAULT NOW(),
    updated_at TIMESTAMPTZ DEFAULT NOW(),
    deleted_at TIMESTAMPTZ,
    UNIQUE(tenant_id, slug)
);

CREATE TRIGGER update_sys_roles_updated_at BEFORE UPDATE ON sys_roles FOR EACH ROW EXECUTE FUNCTION update_updated_at_column();

-- Core Role-Permission Junction
CREATE TABLE IF NOT EXISTS sys_role_permissions (
    role_id UUID REFERENCES sys_roles(id) ON DELETE CASCADE,
    permission_id UUID REFERENCES sys_permissions(id) ON DELETE CASCADE,
    PRIMARY KEY (role_id, permission_id)
);

-- 7. Memberships (User-Tenant-Role)
CREATE TABLE IF NOT EXISTS auth_memberships (
    user_id UUID NOT NULL REFERENCES auth_users(id) ON DELETE CASCADE,
    tenant_id UUID NOT NULL REFERENCES auth_tenants(id) ON DELETE CASCADE,
    role VARCHAR(50) NOT NULL, -- Logical role alias
    role_id UUID REFERENCES sys_roles(id),
    is_active BOOLEAN DEFAULT TRUE,
    created_at TIMESTAMPTZ DEFAULT NOW(),
    updated_at TIMESTAMPTZ DEFAULT NOW(),
    deleted_at TIMESTAMPTZ,
    PRIMARY KEY (user_id, tenant_id)
);

CREATE INDEX idx_memberships_tenant_role ON auth_memberships(tenant_id, role_id);
CREATE TRIGGER update_auth_memberships_updated_at BEFORE UPDATE ON auth_memberships FOR EACH ROW EXECUTE FUNCTION update_updated_at_column();

-- 8. Isolated User Profile Imagery (Privacy Protected)
CREATE TABLE IF NOT EXISTS auth_user_images (
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    user_id UUID NOT NULL REFERENCES auth_users(id) ON DELETE CASCADE,
    url TEXT NOT NULL,
    image_type VARCHAR(20) NOT NULL DEFAULT 'avatar', -- 'avatar', 'cover'
    is_primary BOOLEAN DEFAULT FALSE,
    metadata JSONB DEFAULT '{}'::jsonb,
    created_at TIMESTAMPTZ DEFAULT NOW()
);

CREATE INDEX idx_auth_user_images_user ON auth_user_images(user_id);

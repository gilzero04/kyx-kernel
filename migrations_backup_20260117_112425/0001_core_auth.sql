-- ════════════════════════════════════════════════════════════════════════════
-- Migration: 0001_core_auth.sql
-- Purpose: Core functions, system tables, users, tenants, and memberships
-- Consolidated from: 0001, parts of 0006, 0009, 0010
-- ════════════════════════════════════════════════════════════════════════════

-- ════════════════════════════════════════════════════════════════════════════
-- 1. GLOBAL FUNCTIONS
-- ════════════════════════════════════════════════════════════════════════════

CREATE EXTENSION IF NOT EXISTS "pgcrypto";

CREATE OR REPLACE FUNCTION update_updated_at_column()
RETURNS TRIGGER AS $$
BEGIN
    NEW.updated_at = NOW();
    RETURN NEW;
END;
$$ language 'plpgsql';

-- ════════════════════════════════════════════════════════════════════════════
-- 2. SYSTEM FOUNDATION
-- ════════════════════════════════════════════════════════════════════════════

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
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    key VARCHAR(255) NOT NULL,
    value JSONB NOT NULL,
    description TEXT,
    is_public BOOLEAN DEFAULT FALSE,
    tenant_id UUID NOT NULL,
    scope TEXT NOT NULL DEFAULT 'platform',
    created_at TIMESTAMPTZ DEFAULT NOW(),
    updated_at TIMESTAMPTZ DEFAULT NOW(),
    CONSTRAINT sys_configs_key_tenant_scope_unique UNIQUE (key, tenant_id, scope)
);

CREATE TRIGGER update_sys_configs_updated_at BEFORE UPDATE ON sys_configs FOR EACH ROW EXECUTE FUNCTION update_updated_at_column();

CREATE TABLE IF NOT EXISTS sys_api_keys (
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    tenant_id UUID NOT NULL,
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

-- ════════════════════════════════════════════════════════════════════════════
-- 3. AUTH CORE: USERS
-- ════════════════════════════════════════════════════════════════════════════

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

CREATE TABLE IF NOT EXISTS auth_user_images (
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    user_id UUID NOT NULL REFERENCES auth_users(id) ON DELETE CASCADE,
    url TEXT NOT NULL,
    image_type VARCHAR(20) NOT NULL DEFAULT 'avatar',
    is_primary BOOLEAN DEFAULT FALSE,
    metadata JSONB DEFAULT '{}'::jsonb,
    created_at TIMESTAMPTZ DEFAULT NOW()
);

CREATE INDEX idx_auth_user_images_user ON auth_user_images(user_id);

-- ════════════════════════════════════════════════════════════════════════════
-- 4. TENANT TYPES
-- ════════════════════════════════════════════════════════════════════════════

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

INSERT INTO sys_tenant_types (slug, name, description, icon, is_system) VALUES 
    ('owner', 'System Owner / เจ้าของระบบ', 'Core platform administration', 'shield-check', TRUE),
    ('branch', 'Branch / สาขา', 'Organization branch or division', 'store', TRUE),
    ('vendor', 'Vendor / ผู้ขาย', 'Marketplace seller or merchant', 'shop', TRUE),
    ('supplier', 'Supplier / ผู้จัดจำหน่าย', 'Inventory or goods provider', 'truck', TRUE),
    ('partner', 'Partner / พันธมิตร', 'External business partner', 'handshake', TRUE),
    ('franchise', 'Franchise / แฟรนไชส์', 'Franchise business model', 'award', TRUE),
    ('service_provider', 'Service Provider / ผู้ให้บริการ', 'Third-party service provider', 'briefcase', TRUE),
    ('marketplace', 'Marketplace', 'Aggregator of vendors', 'shopping-bag', TRUE),
    ('platform', 'Platform Owner', 'Multi-tenant platform administrator', 'server', TRUE),
    ('subtenant', 'Subtenant', 'Sub-organization tenant', 'users', TRUE)
ON CONFLICT (slug) DO NOTHING;

-- ════════════════════════════════════════════════════════════════════════════
-- 5. TENANTS (Hierarchical)
-- ════════════════════════════════════════════════════════════════════════════

CREATE TABLE IF NOT EXISTS auth_tenants (
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    parent_id UUID NOT NULL REFERENCES auth_tenants(id) ON DELETE SET NULL,
    tenant_type_id UUID REFERENCES sys_tenant_types(id),
    name VARCHAR(255) NOT NULL,
    slug VARCHAR(255) UNIQUE NOT NULL,
    
    -- Profile & Contact
    contact_email VARCHAR(255),
    contact_phone VARCHAR(50),
    website_url TEXT,
    social_links JSONB DEFAULT '{}'::jsonb,
    address TEXT,
    business_type VARCHAR(100),
    
    -- Domain Management
    custom_domain VARCHAR(255) UNIQUE,
    allow_child_subdomains BOOLEAN DEFAULT FALSE,
    use_parent_subdomain BOOLEAN DEFAULT FALSE,
    domain_verified_at TIMESTAMP WITH TIME ZONE,
    verification_token VARCHAR(64) UNIQUE DEFAULT encode(gen_random_bytes(24), 'base64'),
    
    -- Branding Reference
    branding_id UUID,
    
    -- Config & System
    config JSONB DEFAULT '{}'::jsonb,
    is_active BOOLEAN DEFAULT TRUE,
    created_at TIMESTAMPTZ DEFAULT NOW(),
    updated_at TIMESTAMPTZ DEFAULT NOW(),
    deleted_at TIMESTAMPTZ
);

CREATE INDEX idx_tenants_parent_id ON auth_tenants(parent_id);
CREATE INDEX idx_auth_tenants_custom_domain ON auth_tenants(custom_domain) WHERE custom_domain IS NOT NULL;
CREATE INDEX idx_auth_tenants_branding ON auth_tenants(branding_id);
CREATE TRIGGER update_auth_tenants_updated_at BEFORE UPDATE ON auth_tenants FOR EACH ROW EXECUTE FUNCTION update_updated_at_column();

-- Add FK after sys_configs, sys_api_keys reference auth_tenants
ALTER TABLE sys_configs ADD CONSTRAINT fk_sys_configs_tenant FOREIGN KEY (tenant_id) REFERENCES auth_tenants(id) ON DELETE CASCADE;
ALTER TABLE sys_api_keys ADD CONSTRAINT fk_sys_api_keys_tenant FOREIGN KEY (tenant_id) REFERENCES auth_tenants(id) ON DELETE CASCADE;

-- ════════════════════════════════════════════════════════════════════════════
-- 5.1 TENANT VISIBILITY HELPER FUNCTIONS (RULE 18)
-- These functions enforce strict tenant hierarchy visibility:
-- - Parent can see all descendants
-- - Child cannot see parent or siblings
-- - Cross-network visibility is forbidden
-- ════════════════════════════════════════════════════════════════════════════

-- Get ancestor chain for a tenant (returns array from self to root)
CREATE OR REPLACE FUNCTION get_ancestor_chain(p_tenant_id UUID) 
RETURNS UUID[] AS $$
WITH RECURSIVE chain AS (
    SELECT id, parent_id, ARRAY[id] as path
    FROM auth_tenants 
    WHERE id = p_tenant_id AND deleted_at IS NULL
    UNION ALL
    SELECT t.id, t.parent_id, c.path || t.parent_id
    FROM auth_tenants t
    JOIN chain c ON c.parent_id = t.id
    WHERE t.id != t.parent_id  -- stop at root (self-parent)
    AND t.deleted_at IS NULL
)
SELECT path FROM chain ORDER BY array_length(path, 1) DESC LIMIT 1;
$$ LANGUAGE SQL STABLE;

-- Check if viewer can see target tenant (viewer is ancestor of target)
-- Returns TRUE if: viewer = target OR viewer is in target's ancestor chain
CREATE OR REPLACE FUNCTION can_view_tenant(viewer_id UUID, target_id UUID) 
RETURNS BOOLEAN AS $$
SELECT viewer_id = target_id 
    OR COALESCE(get_ancestor_chain(target_id), ARRAY[]::UUID[]) @> ARRAY[viewer_id]::UUID[];
$$ LANGUAGE SQL STABLE;

-- Get all descendant tenant IDs for a viewer (for parent queries)
-- Returns all tenants that viewer can see (self + all descendants)
CREATE OR REPLACE FUNCTION get_descendant_ids(viewer_id UUID) 
RETURNS UUID[] AS $$
WITH RECURSIVE descendants AS (
    SELECT id FROM auth_tenants 
    WHERE id = viewer_id AND deleted_at IS NULL
    UNION ALL
    SELECT t.id FROM auth_tenants t
    JOIN descendants d ON t.parent_id = d.id
    WHERE t.id != t.parent_id  -- exclude self-parent (root)
    AND t.deleted_at IS NULL
)
SELECT ARRAY_AGG(id) FROM descendants;
$$ LANGUAGE SQL STABLE;

-- ════════════════════════════════════════════════════════════════════════════
-- 6. RBAC BASE: PERMISSIONS & ROLES
-- ════════════════════════════════════════════════════════════════════════════


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
    tenant_id UUID REFERENCES auth_tenants(id) ON DELETE CASCADE, -- NULLABLE for global template roles
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

CREATE TABLE IF NOT EXISTS sys_role_permissions (
    role_id UUID REFERENCES sys_roles(id) ON DELETE CASCADE,
    permission_id UUID REFERENCES sys_permissions(id) ON DELETE CASCADE,
    PRIMARY KEY (role_id, permission_id)
);

CREATE TABLE IF NOT EXISTS sys_tenant_permissions (
    tenant_id UUID NOT NULL REFERENCES auth_tenants(id) ON DELETE CASCADE,
    permission_id UUID NOT NULL REFERENCES sys_permissions(id) ON DELETE CASCADE,
    PRIMARY KEY (tenant_id, permission_id)
);

-- ════════════════════════════════════════════════════════════════════════════
-- 7. MEMBERSHIPS
-- ════════════════════════════════════════════════════════════════════════════

CREATE TABLE IF NOT EXISTS auth_memberships (
    user_id UUID NOT NULL REFERENCES auth_users(id) ON DELETE CASCADE,
    tenant_id UUID NOT NULL REFERENCES auth_tenants(id) ON DELETE CASCADE,
    role VARCHAR(50) NOT NULL,
    role_id UUID REFERENCES sys_roles(id),
    is_active BOOLEAN DEFAULT TRUE,
    created_at TIMESTAMPTZ DEFAULT NOW(),
    updated_at TIMESTAMPTZ DEFAULT NOW(),
    deleted_at TIMESTAMPTZ,
    PRIMARY KEY (user_id, tenant_id)
);

CREATE INDEX idx_memberships_tenant_role ON auth_memberships(tenant_id, role_id);
CREATE TRIGGER update_auth_memberships_updated_at BEFORE UPDATE ON auth_memberships FOR EACH ROW EXECUTE FUNCTION update_updated_at_column();

-- ════════════════════════════════════════════════════════════════════════════
-- 8. USER PREFERENCES
-- ════════════════════════════════════════════════════════════════════════════

CREATE TABLE IF NOT EXISTS user_preferences (
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    user_id UUID NOT NULL REFERENCES auth_users(id) ON DELETE CASCADE,
    preferred_theme_light_id VARCHAR(100),
    preferred_theme_dark_id VARCHAR(100),
    theme_mode VARCHAR(20) DEFAULT 'system',
    locale VARCHAR(10) DEFAULT 'en',
    timezone VARCHAR(50) DEFAULT 'UTC',
    notifications_enabled BOOLEAN DEFAULT TRUE,
    created_at TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    updated_at TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    CONSTRAINT uq_user_preferences_user UNIQUE(user_id)
);

CREATE INDEX idx_user_preferences_user_id ON user_preferences(user_id);

-- ════════════════════════════════════════════════════════════════════════════
-- COMMENTS
-- ════════════════════════════════════════════════════════════════════════════

COMMENT ON TABLE auth_tenants IS 'Hierarchical multi-tenant organization structure';
COMMENT ON COLUMN auth_tenants.parent_id IS 'Self-referencing parent (owner has parent_id = id)';
COMMENT ON COLUMN auth_tenants.custom_domain IS 'Optional custom domain for white-labeling';
COMMENT ON COLUMN auth_tenants.verification_token IS 'Unique token for TXT record verification';
COMMENT ON TABLE user_preferences IS 'User-specific preferences (theme, locale, notifications)';

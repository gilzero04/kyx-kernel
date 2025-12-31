-- ================================================
-- Migration: 0005_feature_modules.sql
-- Purpose: CMS (Pages/Media), Media Library, and Landing Page Logic.
-- ================================================

-- 1. CMS: Pages
CREATE TABLE IF NOT EXISTS sys_pages (
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    tenant_id UUID NOT NULL REFERENCES auth_tenants(id) ON DELETE CASCADE,
    slug VARCHAR(255) NOT NULL,
    title VARCHAR(255) NOT NULL,
    content JSONB DEFAULT '[]'::jsonb,
    is_published BOOLEAN DEFAULT FALSE,
    created_at TIMESTAMPTZ DEFAULT NOW(),
    updated_at TIMESTAMPTZ DEFAULT NOW(),
    deleted_at TIMESTAMPTZ,
    UNIQUE(tenant_id, slug)
);

CREATE INDEX idx_sys_pages_tenant_slug ON sys_pages(tenant_id, slug);
CREATE TRIGGER update_sys_pages_updated_at BEFORE UPDATE ON sys_pages FOR EACH ROW EXECUTE FUNCTION update_updated_at_column();

-- 2. Media & Asset Management
CREATE TABLE IF NOT EXISTS media_folders (
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    tenant_id UUID NOT NULL REFERENCES auth_tenants(id) ON DELETE CASCADE,
    parent_id UUID REFERENCES media_folders(id) ON DELETE CASCADE,
    name VARCHAR(255) NOT NULL,
    created_at TIMESTAMPTZ DEFAULT NOW(),
    updated_at TIMESTAMPTZ DEFAULT NOW()
);

CREATE TABLE IF NOT EXISTS media_assets (
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    tenant_id UUID NOT NULL REFERENCES auth_tenants(id) ON DELETE CASCADE,
    folder_id UUID REFERENCES media_folders(id) ON DELETE SET NULL,
    filename VARCHAR(255) NOT NULL,
    original_name VARCHAR(255) NOT NULL,
    mime_type VARCHAR(100) NOT NULL,
    file_size BIGINT NOT NULL,
    url TEXT NOT NULL,
    visibility VARCHAR(20) NOT NULL DEFAULT 'private',
    metadata JSONB DEFAULT '{}'::jsonb,
    created_at TIMESTAMPTZ DEFAULT NOW(),
    updated_at TIMESTAMPTZ DEFAULT NOW(),
    deleted_at TIMESTAMPTZ
);

CREATE INDEX idx_media_folders_tenant ON media_folders(tenant_id);
CREATE INDEX idx_media_assets_tenant ON media_assets(tenant_id);

CREATE TRIGGER update_media_folders_updated_at BEFORE UPDATE ON media_folders FOR EACH ROW EXECUTE FUNCTION update_updated_at_column();
CREATE TRIGGER update_media_assets_updated_at BEFORE UPDATE ON media_assets FOR EACH ROW EXECUTE FUNCTION update_updated_at_column();

-- 3. Landing & Redirection Logic
CREATE TABLE IF NOT EXISTS sys_tenant_type_role_defaults (
    tenant_type_id UUID NOT NULL REFERENCES sys_tenant_types(id) ON DELETE CASCADE,
    role_slug VARCHAR(50) NOT NULL,
    default_landing_path VARCHAR(255) NOT NULL,
    created_at TIMESTAMPTZ DEFAULT NOW(),
    PRIMARY KEY (tenant_type_id, role_slug)
);

-- Seed defaults for 'owner' (formerly platform/standard)
-- Helper to get ID
CREATE OR REPLACE FUNCTION get_tt_id(slug_in VARCHAR) RETURNS UUID AS $$
    SELECT id FROM sys_tenant_types WHERE slug = slug_in;
$$ LANGUAGE SQL;

INSERT INTO sys_tenant_type_role_defaults (tenant_type_id, role_slug, default_landing_path) VALUES
    (get_tt_id('owner'), 'superadmin', '/admin/dashboard'),
    (get_tt_id('branch'), 'superadmin', '/app/dashboard'),
    (get_tt_id('vendor'), 'superadmin', '/vendor/dashboard')
ON CONFLICT DO NOTHING;

DROP FUNCTION get_tt_id(VARCHAR);

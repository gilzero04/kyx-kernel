-- ================================================
-- Migration: 20251228150000_init_cms_full.sql
-- Purpose: Add Tenant Config, CMS Pages, and Robust Media Library
-- ================================================

-- 1. Add Config to Tenants
ALTER TABLE auth_tenants ADD COLUMN IF NOT EXISTS config JSONB DEFAULT '{}'::jsonb;

-- 2. Seed Tenant Types
INSERT INTO sys_tenant_types (slug, name, description, icon, is_system) VALUES
    ('personal', 'Personal', 'Personal projects and stores', 'user', TRUE),
    ('business', 'Business', 'Business platforms and marketplaces', 'briefcase', TRUE)
ON CONFLICT (slug) DO NOTHING;

-- 3. CMS: Pages (Headless storage for Page Builder)
CREATE TABLE IF NOT EXISTS sys_pages (
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    tenant_id UUID NOT NULL REFERENCES auth_tenants(id) ON DELETE CASCADE,
    slug VARCHAR(255) NOT NULL,
    title VARCHAR(255) NOT NULL,
    content JSONB DEFAULT '[]'::jsonb, -- JSON Tree for Block Builder
    is_published BOOLEAN DEFAULT FALSE,
    created_at TIMESTAMPTZ DEFAULT NOW(),
    updated_at TIMESTAMPTZ DEFAULT NOW(),
    deleted_at TIMESTAMPTZ,
    UNIQUE(tenant_id, slug)
);

-- 4. CMS: Media (Context-aware storage)
CREATE TABLE IF NOT EXISTS sys_media (
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    tenant_id UUID NOT NULL REFERENCES auth_tenants(id) ON DELETE CASCADE,
    filename VARCHAR(255) NOT NULL,
    url TEXT NOT NULL,
    mime_type VARCHAR(100) NOT NULL,
    size_bytes BIGINT NOT NULL,
    alt_text VARCHAR(255),
    
    -- Robust Context Support (e.g., 'builder', 'social_feed', 'avatar', 'vod')
    context VARCHAR(50) NOT NULL DEFAULT 'general',
    
    -- Metadata for app-specific needs (e.g., { duration: 120, resolution: '1080p', blurhash: '...' })
    metadata JSONB DEFAULT '{}'::jsonb,
    
    created_at TIMESTAMPTZ DEFAULT NOW(),
    updated_at TIMESTAMPTZ DEFAULT NOW(),
    deleted_at TIMESTAMPTZ
);

-- Indices
CREATE INDEX IF NOT EXISTS idx_sys_pages_tenant_slug ON sys_pages(tenant_id, slug);
CREATE INDEX IF NOT EXISTS idx_sys_media_tenant_context ON sys_media(tenant_id, context);

-- Triggers for Updated At
DROP TRIGGER IF EXISTS update_sys_pages_updated_at ON sys_pages;
CREATE TRIGGER update_sys_pages_updated_at BEFORE UPDATE ON sys_pages FOR EACH ROW EXECUTE FUNCTION update_updated_at_column();

DROP TRIGGER IF EXISTS update_sys_media_updated_at ON sys_media;
CREATE TRIGGER update_sys_media_updated_at BEFORE UPDATE ON sys_media FOR EACH ROW EXECUTE FUNCTION update_updated_at_column();

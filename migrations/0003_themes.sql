-- ════════════════════════════════════════════════════════════════════════════
-- Migration: 0003_themes.sql
-- Purpose: Theme registry with final schema (slug, type)
-- Consolidated from: 0003, 0011, 0023, 0027, 0028
-- Note: Columns that were created then dropped are NOT included
-- ════════════════════════════════════════════════════════════════════════════

-- ════════════════════════════════════════════════════════════════════════════
-- 1. THEME VISIBILITY TYPE
-- ════════════════════════════════════════════════════════════════════════════

DO $$ BEGIN
    CREATE TYPE theme_visibility AS ENUM ('public', 'private', 'restricted');
EXCEPTION
    WHEN duplicate_object THEN null;
END $$;

-- ════════════════════════════════════════════════════════════════════════════
-- 2. THEME REGISTRY (Final Schema)
-- ════════════════════════════════════════════════════════════════════════════

CREATE TABLE IF NOT EXISTS sys_themes (
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    tenant_id UUID REFERENCES auth_tenants(id) ON DELETE CASCADE,
    
    -- Identity
    name VARCHAR(255) NOT NULL,
    slug VARCHAR(100) UNIQUE,
    type VARCHAR(20) DEFAULT 'dark' CHECK (type IN ('light', 'dark')),
    description TEXT,
    
    -- Metadata
    config JSONB NOT NULL DEFAULT '{}'::jsonb,
    visibility theme_visibility NOT NULL DEFAULT 'private',
    version VARCHAR(20) DEFAULT '1.0.0',
    author VARCHAR(255),
    preview_url TEXT,
    logo_url TEXT,
    
    -- Status
    is_active BOOLEAN DEFAULT TRUE,
    is_system BOOLEAN DEFAULT FALSE,
    
    -- Timestamps
    created_at TIMESTAMPTZ DEFAULT NOW(),
    updated_at TIMESTAMPTZ DEFAULT NOW(),
    
    UNIQUE(tenant_id, name)
);

CREATE TRIGGER update_sys_themes_updated_at BEFORE UPDATE ON sys_themes FOR EACH ROW EXECUTE FUNCTION update_updated_at_column();
CREATE UNIQUE INDEX IF NOT EXISTS idx_sys_themes_slug ON sys_themes(slug);

-- ════════════════════════════════════════════════════════════════════════════
-- 3. SEED DEFAULT THEMES
-- ════════════════════════════════════════════════════════════════════════════

INSERT INTO sys_themes (
    id, name, slug, type, description, version, author,
    preview_url, logo_url, config, visibility, is_active, is_system
) VALUES 
    (
        '6f29f480-e8bb-4fb8-b03f-8c3859544891'::uuid,
        'Kyx Dark',
        'kyx-dark',
        'dark',
        'Night Sea Obsidian',
        '1.0.0',
        'Kyx Technologies Co., Ltd.',
        '/themes/presets/kyx-dark/images/preview.png',
        '/themes/presets/kyx-dark/images/logo.png',
        '{
            "splash": {"type": "text", "text": "KYX PLATFORM", "subtext": "Initializing Platform..."},
            "background": {"particleCount": 15, "showWaves": true},
            "layout": {"cardStyle": "glass", "contentWidth": "normal"},
            "customText": {"footer_message": "Secure. Stable. Scalable."}
        }'::jsonb,
        'public',
        true,
        true
    ),
    (
        'b7859090-98fe-4e49-8d2b-17d402d0d0dd'::uuid,
        'Kyx Light',
        'kyx-light',
        'light',
        'Sunrise Pearl Azure',
        '1.0.0',
        'Kyx Technologies Co., Ltd.',
        '/themes/presets/kyx-light/images/preview.png',
        '/themes/presets/kyx-light/images/logo.png',
        '{
            "splash": {"type": "text", "text": "KYX PLATFORM", "subtext": "Initializing Platform..."},
            "background": {"particleCount": 15, "showWaves": true},
            "layout": {"cardStyle": "glass", "contentWidth": "normal"},
            "customText": {"footer_message": "Secure. Stable. Scalable."}
        }'::jsonb,
        'public',
        true,
        true
    )
ON CONFLICT (id) DO UPDATE SET
    name = EXCLUDED.name,
    slug = EXCLUDED.slug,
    type = EXCLUDED.type,
    description = EXCLUDED.description,
    config = EXCLUDED.config,
    is_system = EXCLUDED.is_system,
    updated_at = NOW();

-- ════════════════════════════════════════════════════════════════════════════
-- COMMENTS
-- ════════════════════════════════════════════════════════════════════════════

COMMENT ON TABLE sys_themes IS 'Theme registry - stores theme configurations';
COMMENT ON COLUMN sys_themes.slug IS 'Human-readable identifier (e.g. kyx-dark, pixco-ocean)';
COMMENT ON COLUMN sys_themes.type IS 'Theme mode: light or dark';
COMMENT ON COLUMN sys_themes.is_system IS 'True for official Kyx themes bundled with the platform';

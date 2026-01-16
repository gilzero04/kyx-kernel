-- Migration: 0028_seed_default_themes.sql
-- Purpose: Seed default Kyx themes and rename code column to slug
-- Author: AI-generated per theme system design
-- Date: 2026-01-16
--
-- Changes:
-- 1. Rename column 'code' to 'slug' for human-readable theme identifiers
-- 2. Add 'type' column for light/dark mode
-- 3. Seed kyx-dark and kyx-light themes
-- 4. Set default themes for owner branding

-- ============================================================================
-- 1. Rename 'code' column to 'slug' if not already done
-- ============================================================================

DO $$
BEGIN
    IF EXISTS (
        SELECT 1 FROM information_schema.columns 
        WHERE table_name = 'sys_themes' AND column_name = 'code'
    ) AND NOT EXISTS (
        SELECT 1 FROM information_schema.columns 
        WHERE table_name = 'sys_themes' AND column_name = 'slug'
    ) THEN
        ALTER TABLE sys_themes RENAME COLUMN code TO slug;
    END IF;
END $$;

COMMENT ON COLUMN sys_themes.slug IS 'Human-readable identifier (e.g. kyx-dark, pixco-ocean)';

-- ============================================================================
-- 2. Add 'type' column if not exists (light/dark)
-- ============================================================================

DO $$
BEGIN
    IF NOT EXISTS (
        SELECT 1 FROM information_schema.columns 
        WHERE table_name = 'sys_themes' AND column_name = 'type'
    ) THEN
        ALTER TABLE sys_themes ADD COLUMN type VARCHAR(20) DEFAULT 'dark' 
            CHECK (type IN ('light', 'dark'));
    END IF;
END $$;

COMMENT ON COLUMN sys_themes.type IS 'Theme mode: light or dark';

-- ============================================================================
-- 3. Seed Kyx Dark theme
-- ============================================================================

INSERT INTO sys_themes (
    id,
    name,
    slug,
    type,
    description,
    version,
    author,
    preview_url,
    logo_url,
    config,
    visibility,
    is_active,
    is_system
) VALUES (
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
        "splash": {
            "type": "text",
            "text": "KYX PLATFORM",
            "subtext": "Initializing Platform..."
        },
        "background": {
            "particleCount": 15,
            "showWaves": true
        },
        "layout": {
            "cardStyle": "glass",
            "contentWidth": "normal"
        },
        "customText": {
            "footer_message": "Secure. Stable. Scalable."
        }
    }'::jsonb,
    'public',
    true,
    true
) ON CONFLICT (id) DO UPDATE SET
    name = EXCLUDED.name,
    slug = EXCLUDED.slug,
    type = EXCLUDED.type,
    description = EXCLUDED.description,
    version = EXCLUDED.version,
    author = EXCLUDED.author,
    config = EXCLUDED.config,
    is_system = EXCLUDED.is_system,
    updated_at = NOW();

-- ============================================================================
-- 4. Seed Kyx Light theme
-- ============================================================================

INSERT INTO sys_themes (
    id,
    name,
    slug,
    type,
    description,
    version,
    author,
    preview_url,
    logo_url,
    config,
    visibility,
    is_active,
    is_system
) VALUES (
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
        "splash": {
            "type": "text",
            "text": "KYX PLATFORM",
            "subtext": "Initializing Platform..."
        },
        "background": {
            "particleCount": 15,
            "showWaves": true
        },
        "layout": {
            "cardStyle": "glass",
            "contentWidth": "normal"
        },
        "customText": {
            "footer_message": "Secure. Stable. Scalable."
        }
    }'::jsonb,
    'public',
    true,
    true
) ON CONFLICT (id) DO UPDATE SET
    name = EXCLUDED.name,
    slug = EXCLUDED.slug,
    type = EXCLUDED.type,
    description = EXCLUDED.description,
    version = EXCLUDED.version,
    author = EXCLUDED.author,
    config = EXCLUDED.config,
    is_system = EXCLUDED.is_system,
    updated_at = NOW();

-- ============================================================================
-- 5. Set default themes for owner branding if not set
-- ============================================================================

UPDATE sys_brandings b SET 
    theme_light_id = 'b7859090-98fe-4e49-8d2b-17d402d0d0dd'::uuid,
    theme_dark_id = '6f29f480-e8bb-4fb8-b03f-8c3859544891'::uuid,
    theme_workspace_light_id = COALESCE(theme_workspace_light_id, 'b7859090-98fe-4e49-8d2b-17d402d0d0dd'::uuid),
    theme_workspace_dark_id = COALESCE(theme_workspace_dark_id, '6f29f480-e8bb-4fb8-b03f-8c3859544891'::uuid),
    theme_app_light_id = COALESCE(theme_app_light_id, 'b7859090-98fe-4e49-8d2b-17d402d0d0dd'::uuid),
    theme_app_dark_id = COALESCE(theme_app_dark_id, '6f29f480-e8bb-4fb8-b03f-8c3859544891'::uuid),
    updated_at = NOW()
FROM auth_tenants t
WHERE b.tenant_id = t.id
  AND t.parent_id = t.id  -- Owner tenant only
  AND b.theme_light_id IS NULL;

-- ============================================================================
-- 6. Comments
-- ============================================================================

COMMENT ON COLUMN sys_themes.id IS 
'Theme unique identifier (UUID)';
COMMENT ON COLUMN sys_themes.slug IS 
'Human-readable identifier from manifest.json (e.g. kyx-dark, pixco-ocean)';
COMMENT ON COLUMN sys_themes.is_system IS 
'True for official Kyx themes bundled with the platform';

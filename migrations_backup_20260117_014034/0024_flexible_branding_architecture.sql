-- Migration: 0024_flexible_branding_architecture.sql
-- Purpose: Consolidate branding into sys_brandings ONLY (remove redundancy)
-- Author: AI-generated per architecture design
-- Date: 2026-01-08
-- BREAKING CHANGE: Moves branding away from auth_tenants columns and sys_configs

-- ============================================================================
-- 1. Create sys_brandings table (Single Source of Truth for branding)
-- ============================================================================

CREATE TABLE IF NOT EXISTS sys_brandings (
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    
    -- ─── Identity ───────────────────────────────────────────────────────────
    name VARCHAR(255) NOT NULL,           -- Internal name for this branding config
    description TEXT,
    
    -- ─── Visual Assets ──────────────────────────────────────────────────────
    logo_light_url TEXT,           -- Logo for light mode
    logo_dark_url TEXT,            -- Logo for dark mode
    favicon_url TEXT,              -- Browser favicon
    icon_app_url TEXT,             -- PWA/App icon
    splash_image_url TEXT,         -- Splash screen image
    
    -- ─── Colors ─────────────────────────────────────────────────────────────
    primary_color VARCHAR(20),     -- e.g., #4f46e5
    secondary_color VARCHAR(20),
    accent_color VARCHAR(20),
    
    -- ─── Text/Display ───────────────────────────────────────────────────────
    app_name VARCHAR(255),         -- Display name
    splash_text VARCHAR(255),      -- Splash screen main text
    splash_subtext VARCHAR(255),   -- Splash screen subtitle
    tagline VARCHAR(255),          -- Company tagline
    
    -- ─── Theme References ───────────────────────────────────────────────────
    theme_light_id UUID REFERENCES sys_themes(id) ON DELETE SET NULL,
    theme_dark_id UUID REFERENCES sys_themes(id) ON DELETE SET NULL,
    
    -- ─── Extensible Metadata ────────────────────────────────────────────────
    metadata JSONB DEFAULT '{}',   -- For future extensions
    
    -- ─── Timestamps ─────────────────────────────────────────────────────────
    created_at TIMESTAMPTZ DEFAULT NOW(),
    updated_at TIMESTAMPTZ DEFAULT NOW()
);

-- Index for lookups
CREATE INDEX IF NOT EXISTS idx_sys_brandings_name ON sys_brandings(name);

-- ============================================================================
-- 2. Add branding_id to auth_tenants (FK relationship)
-- ============================================================================

DO $$
BEGIN
    IF NOT EXISTS (
        SELECT 1 FROM information_schema.columns 
        WHERE table_name = 'auth_tenants' AND column_name = 'branding_id'
    ) THEN
        ALTER TABLE auth_tenants 
        ADD COLUMN branding_id UUID REFERENCES sys_brandings(id) ON DELETE SET NULL;
    END IF;
END $$;

CREATE INDEX IF NOT EXISTS idx_auth_tenants_branding ON auth_tenants(branding_id);

-- ============================================================================
-- 3. Migrate existing branding data from auth_tenants to sys_brandings
-- ============================================================================

-- For each tenant with branding data, create a sys_brandings record
DO $$
DECLARE
    tenant_row RECORD;
    new_branding_id UUID;
BEGIN
    FOR tenant_row IN 
        SELECT id, name, logo_url, logo_dark_url, favicon_url, icon_app_url,
               primary_color, secondary_color, accent_color, app_name_override
        FROM auth_tenants 
        WHERE (logo_url IS NOT NULL OR primary_color IS NOT NULL OR app_name_override IS NOT NULL)
          AND branding_id IS NULL
    LOOP
        -- Create branding record
        INSERT INTO sys_brandings (
            name, logo_light_url, logo_dark_url, favicon_url, icon_app_url,
            primary_color, secondary_color, accent_color, app_name
        ) VALUES (
            tenant_row.name || ' Branding',
            tenant_row.logo_url,
            tenant_row.logo_dark_url,
            tenant_row.favicon_url,
            tenant_row.icon_app_url,
            tenant_row.primary_color,
            tenant_row.secondary_color,
            tenant_row.accent_color,
            tenant_row.app_name_override
        ) RETURNING id INTO new_branding_id;
        
        -- Link tenant to new branding
        UPDATE auth_tenants SET branding_id = new_branding_id WHERE id = tenant_row.id;
    END LOOP;
END $$;

-- ============================================================================
-- 4. Migrate sys_configs branding keys to Owner's sys_brandings
-- ============================================================================

DO $$
DECLARE
    owner_tenant_id UUID;
    owner_branding_id UUID;
    v_app_name VARCHAR(255);
    v_logo_url TEXT;
    v_logo_dark_url TEXT;
    v_splash_text VARCHAR(255);
    v_splash_subtext VARCHAR(255);
    v_favicon_url TEXT;
    v_icon_app_url TEXT;
    v_theme_light UUID;
    v_theme_dark UUID;
BEGIN
    -- Get owner tenant
    SELECT id INTO owner_tenant_id FROM auth_tenants WHERE parent_id = id LIMIT 1;
    
    IF owner_tenant_id IS NOT NULL THEN
        -- Get values from sys_configs
        SELECT value::text INTO v_app_name FROM sys_configs WHERE key = 'branding_app_name' AND scope = 'global' LIMIT 1;
        SELECT value::text INTO v_logo_url FROM sys_configs WHERE key = 'branding_logo_url' AND scope = 'global' LIMIT 1;
        SELECT value::text INTO v_logo_dark_url FROM sys_configs WHERE key = 'branding_logo_dark_url' AND scope = 'global' LIMIT 1;
        SELECT value::text INTO v_splash_text FROM sys_configs WHERE key = 'branding_splash_text' AND scope = 'global' LIMIT 1;
        SELECT value::text INTO v_splash_subtext FROM sys_configs WHERE key = 'branding_splash_init_text' AND scope = 'global' LIMIT 1;
        SELECT value::text INTO v_favicon_url FROM sys_configs WHERE key = 'branding_favicon_console' AND scope = 'global' LIMIT 1;
        SELECT value::text INTO v_icon_app_url FROM sys_configs WHERE key = 'branding_icon_app' AND scope = 'global' LIMIT 1;
        SELECT (value::text)::UUID INTO v_theme_light FROM sys_configs WHERE key = 'theme_light_id' AND scope = 'global' LIMIT 1;
        SELECT (value::text)::UUID INTO v_theme_dark FROM sys_configs WHERE key = 'theme_dark_id' AND scope = 'global' LIMIT 1;
        
        -- Check if owner already has branding
        SELECT branding_id INTO owner_branding_id FROM auth_tenants WHERE id = owner_tenant_id;
        
        IF owner_branding_id IS NOT NULL THEN
            -- Update existing branding with sys_configs values (if they have values)
            UPDATE sys_brandings SET
                app_name = COALESCE(NULLIF(TRIM(BOTH '"' FROM v_app_name), ''), app_name),
                logo_light_url = COALESCE(NULLIF(TRIM(BOTH '"' FROM v_logo_url), ''), logo_light_url),
                logo_dark_url = COALESCE(NULLIF(TRIM(BOTH '"' FROM v_logo_dark_url), ''), logo_dark_url),
                splash_text = COALESCE(NULLIF(TRIM(BOTH '"' FROM v_splash_text), ''), splash_text),
                splash_subtext = COALESCE(NULLIF(TRIM(BOTH '"' FROM v_splash_subtext), ''), splash_subtext),
                favicon_url = COALESCE(NULLIF(TRIM(BOTH '"' FROM v_favicon_url), ''), favicon_url),
                icon_app_url = COALESCE(NULLIF(TRIM(BOTH '"' FROM v_icon_app_url), ''), icon_app_url),
                theme_light_id = COALESCE(v_theme_light, theme_light_id),
                theme_dark_id = COALESCE(v_theme_dark, theme_dark_id),
                updated_at = NOW()
            WHERE id = owner_branding_id;
        ELSE
            -- Create new branding for owner
            INSERT INTO sys_brandings (
                name, app_name, logo_light_url, logo_dark_url, 
                splash_text, splash_subtext, favicon_url, icon_app_url,
                theme_light_id, theme_dark_id
            ) VALUES (
                'Platform Branding',
                NULLIF(TRIM(BOTH '"' FROM v_app_name), ''),
                NULLIF(TRIM(BOTH '"' FROM v_logo_url), ''),
                NULLIF(TRIM(BOTH '"' FROM v_logo_dark_url), ''),
                NULLIF(TRIM(BOTH '"' FROM v_splash_text), ''),
                NULLIF(TRIM(BOTH '"' FROM v_splash_subtext), ''),
                NULLIF(TRIM(BOTH '"' FROM v_favicon_url), ''),
                NULLIF(TRIM(BOTH '"' FROM v_icon_app_url), ''),
                v_theme_light,
                v_theme_dark
            ) RETURNING id INTO owner_branding_id;
            
            UPDATE auth_tenants SET branding_id = owner_branding_id WHERE id = owner_tenant_id;
        END IF;
    END IF;
END $$;

-- ============================================================================
-- 5. Delete branding keys from sys_configs (no longer needed)
-- ============================================================================

DELETE FROM sys_configs WHERE key LIKE 'branding_%';
DELETE FROM sys_configs WHERE key IN ('theme_light_id', 'theme_dark_id') AND scope = 'global';

-- ============================================================================
-- 6. Drop redundant columns from auth_tenants (after migration)
-- ============================================================================

ALTER TABLE auth_tenants 
    DROP COLUMN IF EXISTS logo_url,
    DROP COLUMN IF EXISTS logo_dark_url,
    DROP COLUMN IF EXISTS favicon_url,
    DROP COLUMN IF EXISTS icon_app_url,
    DROP COLUMN IF EXISTS primary_color,
    DROP COLUMN IF EXISTS secondary_color,
    DROP COLUMN IF EXISTS accent_color,
    DROP COLUMN IF EXISTS app_name_override;

-- ============================================================================
-- 7. Add new tenant types (extensibility)
-- ============================================================================

INSERT INTO sys_tenant_types (slug, name, description, is_active)
VALUES 
    ('branch', 'Branch', 'Branch office or location', true),
    ('vendor', 'Vendor', 'Vendor or supplier partner', true),
    ('supplier', 'Supplier', 'Supply chain partner', true),
    ('subtenant', 'Subtenant', 'Sub-organization tenant', true)
ON CONFLICT (slug) DO NOTHING;

-- ============================================================================
-- 8. Create workspace context table
-- ============================================================================

CREATE TABLE IF NOT EXISTS sys_workspace_contexts (
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    user_id UUID NOT NULL REFERENCES auth_users(id) ON DELETE CASCADE,
    session_id VARCHAR(255),
    active_tenant_id UUID NOT NULL REFERENCES auth_tenants(id) ON DELETE CASCADE,
    ui_overrides JSONB DEFAULT '{}',
    created_at TIMESTAMPTZ DEFAULT NOW(),
    updated_at TIMESTAMPTZ DEFAULT NOW(),
    CONSTRAINT unique_user_session UNIQUE (user_id, session_id)
);

CREATE INDEX IF NOT EXISTS idx_workspace_contexts_user ON sys_workspace_contexts(user_id);
CREATE INDEX IF NOT EXISTS idx_workspace_contexts_tenant ON sys_workspace_contexts(active_tenant_id);

-- ============================================================================
-- 9. Triggers for updated_at
-- ============================================================================

CREATE OR REPLACE FUNCTION update_updated_at_column()
RETURNS TRIGGER AS $$
BEGIN
    NEW.updated_at = NOW();
    RETURN NEW;
END;
$$ language 'plpgsql';

DROP TRIGGER IF EXISTS sys_brandings_updated_at ON sys_brandings;
CREATE TRIGGER sys_brandings_updated_at
    BEFORE UPDATE ON sys_brandings
    FOR EACH ROW EXECUTE FUNCTION update_updated_at_column();

DROP TRIGGER IF EXISTS sys_workspace_contexts_updated_at ON sys_workspace_contexts;
CREATE TRIGGER sys_workspace_contexts_updated_at
    BEFORE UPDATE ON sys_workspace_contexts
    FOR EACH ROW EXECUTE FUNCTION update_updated_at_column();

-- ============================================================================
-- 10. Comments
-- ============================================================================

COMMENT ON TABLE sys_brandings IS 'Single source of truth for branding configurations';
COMMENT ON TABLE sys_workspace_contexts IS 'Tracks active tenant context per user session';
COMMENT ON COLUMN auth_tenants.branding_id IS 'Reference to branding configuration';

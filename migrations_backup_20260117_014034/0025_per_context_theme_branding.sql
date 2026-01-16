-- Migration: 0025_per_context_theme_branding.sql
-- Purpose: Add per-context Theme (3 contexts) and Branding (2 contexts) support
-- Author: AI-generated per architecture design
-- Date: 2026-01-12
-- 
-- Theme Contexts: Console, Workspace, App (3 separate)
-- Branding Contexts: Console, Workspace (2 separate, App shares Workspace)

-- ============================================================================
-- 1. Add context column to sys_brandings
-- ============================================================================

DO $$
BEGIN
    IF NOT EXISTS (
        SELECT 1 FROM information_schema.columns 
        WHERE table_name = 'sys_brandings' AND column_name = 'context'
    ) THEN
        ALTER TABLE sys_brandings ADD COLUMN context VARCHAR(20) 
            DEFAULT 'workspace' CHECK (context IN ('console', 'workspace'));
    END IF;
END $$;

-- ============================================================================
-- 2. Add tenant_id to sys_brandings (for direct lookup)
-- ============================================================================

DO $$
BEGIN
    IF NOT EXISTS (
        SELECT 1 FROM information_schema.columns 
        WHERE table_name = 'sys_brandings' AND column_name = 'tenant_id'
    ) THEN
        ALTER TABLE sys_brandings ADD COLUMN tenant_id UUID REFERENCES auth_tenants(id) ON DELETE CASCADE;
    END IF;
END $$;

-- ============================================================================
-- 3. Add per-context theme columns
-- ============================================================================

-- Console themes
DO $$
BEGIN
    IF NOT EXISTS (SELECT 1 FROM information_schema.columns WHERE table_name = 'sys_brandings' AND column_name = 'theme_console_light_id') THEN
        ALTER TABLE sys_brandings ADD COLUMN theme_console_light_id UUID REFERENCES sys_themes(id) ON DELETE SET NULL;
    END IF;
    IF NOT EXISTS (SELECT 1 FROM information_schema.columns WHERE table_name = 'sys_brandings' AND column_name = 'theme_console_dark_id') THEN
        ALTER TABLE sys_brandings ADD COLUMN theme_console_dark_id UUID REFERENCES sys_themes(id) ON DELETE SET NULL;
    END IF;
END $$;

-- Workspace themes
DO $$
BEGIN
    IF NOT EXISTS (SELECT 1 FROM information_schema.columns WHERE table_name = 'sys_brandings' AND column_name = 'theme_workspace_light_id') THEN
        ALTER TABLE sys_brandings ADD COLUMN theme_workspace_light_id UUID REFERENCES sys_themes(id) ON DELETE SET NULL;
    END IF;
    IF NOT EXISTS (SELECT 1 FROM information_schema.columns WHERE table_name = 'sys_brandings' AND column_name = 'theme_workspace_dark_id') THEN
        ALTER TABLE sys_brandings ADD COLUMN theme_workspace_dark_id UUID REFERENCES sys_themes(id) ON DELETE SET NULL;
    END IF;
END $$;

-- App themes
DO $$
BEGIN
    IF NOT EXISTS (SELECT 1 FROM information_schema.columns WHERE table_name = 'sys_brandings' AND column_name = 'theme_app_light_id') THEN
        ALTER TABLE sys_brandings ADD COLUMN theme_app_light_id UUID REFERENCES sys_themes(id) ON DELETE SET NULL;
    END IF;
    IF NOT EXISTS (SELECT 1 FROM information_schema.columns WHERE table_name = 'sys_brandings' AND column_name = 'theme_app_dark_id') THEN
        ALTER TABLE sys_brandings ADD COLUMN theme_app_dark_id UUID REFERENCES sys_themes(id) ON DELETE SET NULL;
    END IF;
END $$;

-- ============================================================================
-- 4. Create unique index for tenant + context
-- ============================================================================

CREATE UNIQUE INDEX IF NOT EXISTS idx_sys_brandings_tenant_context 
    ON sys_brandings(tenant_id, context) WHERE tenant_id IS NOT NULL;

-- ============================================================================
-- 5. Migrate existing data
-- ============================================================================

-- 5a. Set tenant_id from auth_tenants.branding_id relationship
UPDATE sys_brandings b SET tenant_id = t.id 
FROM auth_tenants t WHERE t.branding_id = b.id AND b.tenant_id IS NULL;

-- 5b. Default existing branding to 'workspace' context
UPDATE sys_brandings SET context = 'workspace' WHERE context IS NULL;

-- 5c. Copy existing theme_light_id/theme_dark_id to workspace context
UPDATE sys_brandings SET 
    theme_workspace_light_id = COALESCE(theme_workspace_light_id, theme_light_id),
    theme_workspace_dark_id = COALESCE(theme_workspace_dark_id, theme_dark_id)
WHERE theme_workspace_light_id IS NULL OR theme_workspace_dark_id IS NULL;

-- 5d. For App, use same as Workspace (shared)
UPDATE sys_brandings SET 
    theme_app_light_id = COALESCE(theme_app_light_id, theme_workspace_light_id),
    theme_app_dark_id = COALESCE(theme_app_dark_id, theme_workspace_dark_id)
WHERE theme_app_light_id IS NULL OR theme_app_dark_id IS NULL;

-- ============================================================================
-- 6. Create console branding records for Owner and Business tenants
-- ============================================================================

INSERT INTO sys_brandings (
    tenant_id, context, name, description,
    logo_light_url, logo_dark_url, favicon_url, icon_app_url, splash_image_url,
    primary_color, secondary_color, accent_color,
    app_name, splash_text, splash_subtext, tagline,
    theme_console_light_id, theme_console_dark_id,
    metadata
)
SELECT 
    t.id,
    'console',
    b.name || ' (Console)',
    'Console branding for ' || t.name,
    b.logo_light_url,
    b.logo_dark_url,
    b.favicon_url,
    b.icon_app_url,
    b.splash_image_url,
    b.primary_color,
    b.secondary_color,
    b.accent_color,
    b.app_name,
    b.splash_text,
    b.splash_subtext,
    b.tagline,
    b.theme_light_id,  -- Use existing as console default
    b.theme_dark_id,
    '{}'::jsonb
FROM auth_tenants t
JOIN sys_brandings b ON t.branding_id = b.id
JOIN sys_tenant_types tt ON t.tenant_type_id = tt.id
WHERE tt.slug IN ('owner', 'business')  -- Only Owner and Business get Console branding
AND NOT EXISTS (
    SELECT 1 FROM sys_brandings 
    WHERE tenant_id = t.id AND context = 'console'
);

-- ============================================================================
-- 7. Add comments
-- ============================================================================

COMMENT ON COLUMN sys_brandings.context IS 'Branding context: console or workspace (app shares workspace)';
COMMENT ON COLUMN sys_brandings.tenant_id IS 'Direct reference to owning tenant';
COMMENT ON COLUMN sys_brandings.theme_console_light_id IS 'Theme for Console in light mode';
COMMENT ON COLUMN sys_brandings.theme_console_dark_id IS 'Theme for Console in dark mode';
COMMENT ON COLUMN sys_brandings.theme_workspace_light_id IS 'Theme for Workspace in light mode';
COMMENT ON COLUMN sys_brandings.theme_workspace_dark_id IS 'Theme for Workspace in dark mode';
COMMENT ON COLUMN sys_brandings.theme_app_light_id IS 'Theme for App in light mode';
COMMENT ON COLUMN sys_brandings.theme_app_dark_id IS 'Theme for App in dark mode';

-- ============================================================================
-- 8. Update trigger for updated_at (if not exists)
-- ============================================================================

CREATE OR REPLACE FUNCTION update_sys_brandings_updated_at()
RETURNS TRIGGER AS $$
BEGIN
    NEW.updated_at = NOW();
    RETURN NEW;
END;
$$ language 'plpgsql';

DROP TRIGGER IF EXISTS sys_brandings_updated_at_trigger ON sys_brandings;
CREATE TRIGGER sys_brandings_updated_at_trigger
    BEFORE UPDATE ON sys_brandings
    FOR EACH ROW EXECUTE FUNCTION update_sys_brandings_updated_at();

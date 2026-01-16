-- Migration: 0027_cleanup_console_theme_columns.sql
-- Purpose: Remove redundant theme_console_* columns, clean up naming, and sync console → workspace/app
-- Author: AI-generated per architecture simplification
-- Date: 2026-01-16
--
-- Design decision:
-- - theme_light_id/theme_dark_id are console defaults (also used as fallback)
-- - theme_workspace_* and theme_app_* are context-specific overrides
-- - `context` column is kept to differentiate Console vs Workspace branding
-- - Names should NOT include "(Console)" - use `context` column instead
-- - Workspace/App inherit NULL fields from Console

-- ============================================================================
-- 1. Drop redundant console theme columns
-- ============================================================================

ALTER TABLE sys_brandings 
    DROP COLUMN IF EXISTS theme_console_light_id,
    DROP COLUMN IF EXISTS theme_console_dark_id;

-- ============================================================================
-- 2. Clean up names - remove "(Console)" suffix, use context column instead
-- ============================================================================

UPDATE sys_brandings 
SET name = TRIM(REPLACE(REPLACE(name, ' (Console)', ''), '(Console)', ''))
WHERE name LIKE '%(Console)%';

-- ============================================================================
-- 3. Sync console values to workspace/app for NULL fields
--    If workspace/app branding has NULL values, copy from console branding
-- ============================================================================

-- For each tenant with multiple branding records (console + workspace),
-- copy NULL fields from console to workspace
-- Note: Colors come from Theme, not Branding (so not synced here)
UPDATE sys_brandings ws
SET 
    -- Visual Assets
    logo_light_url = COALESCE(ws.logo_light_url, con.logo_light_url),
    logo_dark_url = COALESCE(ws.logo_dark_url, con.logo_dark_url),
    favicon_url = COALESCE(ws.favicon_url, con.favicon_url),
    icon_app_url = COALESCE(ws.icon_app_url, con.icon_app_url),
    splash_image_url = COALESCE(ws.splash_image_url, con.splash_image_url),
    -- Text
    app_name = COALESCE(ws.app_name, con.app_name),
    splash_text = COALESCE(ws.splash_text, con.splash_text),
    splash_subtext = COALESCE(ws.splash_subtext, con.splash_subtext),
    tagline = COALESCE(ws.tagline, con.tagline),
    -- Themes (use console as default if workspace themes are null)
    theme_light_id = COALESCE(ws.theme_light_id, con.theme_light_id),
    theme_dark_id = COALESCE(ws.theme_dark_id, con.theme_dark_id),
    theme_workspace_light_id = COALESCE(ws.theme_workspace_light_id, con.theme_light_id),
    theme_workspace_dark_id = COALESCE(ws.theme_workspace_dark_id, con.theme_dark_id),
    theme_app_light_id = COALESCE(ws.theme_app_light_id, con.theme_light_id),
    theme_app_dark_id = COALESCE(ws.theme_app_dark_id, con.theme_dark_id),
    updated_at = NOW()
FROM sys_brandings con
WHERE ws.context = 'workspace' 
  AND con.context = 'console'
  AND ws.tenant_id = con.tenant_id
  AND ws.tenant_id IS NOT NULL;

-- ============================================================================
-- 4. Update comments
-- ============================================================================

COMMENT ON COLUMN sys_brandings.context IS 'Branding context: console or workspace (use this, not name suffix)';
COMMENT ON COLUMN sys_brandings.theme_light_id IS 'Default/Console theme for light mode';
COMMENT ON COLUMN sys_brandings.theme_dark_id IS 'Default/Console theme for dark mode';
COMMENT ON COLUMN sys_brandings.theme_workspace_light_id IS 'Workspace theme override for light mode (fallback to theme_light_id)';
COMMENT ON COLUMN sys_brandings.theme_workspace_dark_id IS 'Workspace theme override for dark mode (fallback to theme_dark_id)';
COMMENT ON COLUMN sys_brandings.theme_app_light_id IS 'App theme override for light mode (fallback to theme_light_id)';
COMMENT ON COLUMN sys_brandings.theme_app_dark_id IS 'App theme override for dark mode (fallback to theme_dark_id)';

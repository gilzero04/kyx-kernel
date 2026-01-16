-- ================================================
-- Migration: 0011_cleanup_redundant_theme_cols.sql
-- Purpose: Remove redundant theme columns from auth_tenants.
-- Note: Theme configuration is now handled via sys_configs with scoping.
-- ================================================

ALTER TABLE auth_tenants 
DROP COLUMN IF EXISTS active_theme_id,
DROP COLUMN IF EXISTS workspace_theme_light_id,
DROP COLUMN IF EXISTS workspace_theme_dark_id,
DROP COLUMN IF EXISTS app_theme_light_id,
DROP COLUMN IF EXISTS app_theme_dark_id;

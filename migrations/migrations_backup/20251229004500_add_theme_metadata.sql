-- Migration: 20251229004500_add_theme_metadata.sql
-- Purpose: Add author, preview_url, and logo_url columns to sys_themes for richer theme metadata.

ALTER TABLE sys_themes
ADD COLUMN IF NOT EXISTS author VARCHAR(255),
ADD COLUMN IF NOT EXISTS preview_url TEXT,
ADD COLUMN IF NOT EXISTS logo_url TEXT;

COMMENT ON COLUMN sys_themes.author IS 'Theme creator/manufacturer name';
COMMENT ON COLUMN sys_themes.preview_url IS 'URL path to theme preview image';
COMMENT ON COLUMN sys_themes.logo_url IS 'URL path to theme logo image';

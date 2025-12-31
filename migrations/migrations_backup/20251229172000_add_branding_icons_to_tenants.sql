-- ================================================
-- Migration: 20251229172000_add_branding_icons_to_tenants.sql
-- Purpose: Add favicon and app icon support to organized branding
-- ================================================

ALTER TABLE auth_tenants 
ADD COLUMN IF NOT EXISTS favicon_url VARCHAR(255),
ADD COLUMN IF NOT EXISTS icon_app_url VARCHAR(255);

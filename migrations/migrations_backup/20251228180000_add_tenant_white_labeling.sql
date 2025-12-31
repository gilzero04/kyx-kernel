-- =====================
-- Migration: 20251228180000_add_tenant_white_labeling.sql
-- Purpose: Add white-labeling support to tenants
-- =====================

ALTER TABLE auth_tenants
ADD COLUMN IF NOT EXISTS primary_color VARCHAR(7),
ADD COLUMN IF NOT EXISTS secondary_color VARCHAR(7),
ADD COLUMN IF NOT EXISTS accent_color VARCHAR(7),
ADD COLUMN IF NOT EXISTS app_name_override VARCHAR(100);

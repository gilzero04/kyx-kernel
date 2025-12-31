-- ================================================
-- Migration: 20251226193900_add_tenant_logos.sql
-- Purpose: Add logo support to tenants
-- ================================================

ALTER TABLE auth_tenants 
ADD COLUMN IF NOT EXISTS logo_url VARCHAR(255),
ADD COLUMN IF NOT EXISTS logo_dark_url VARCHAR(255);

-- =====================
-- Migration: 20251228183000_add_tenant_profile_fields.sql
-- Purpose: Add social and contact profile fields to tenants
-- =====================

ALTER TABLE auth_tenants
ADD COLUMN IF NOT EXISTS contact_email VARCHAR(255),
ADD COLUMN IF NOT EXISTS contact_phone VARCHAR(50),
ADD COLUMN IF NOT EXISTS website_url VARCHAR(255),
ADD COLUMN IF NOT EXISTS social_links JSONB DEFAULT '{}'::jsonb,
ADD COLUMN IF NOT EXISTS address TEXT,
ADD COLUMN IF NOT EXISTS business_type VARCHAR(100);

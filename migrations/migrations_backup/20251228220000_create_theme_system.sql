-- =====================
-- Migration: 20251228220000_create_theme_system.sql
-- Purpose: Advanced Theme System with Network/Tenant scoping (WP Multisite style)
-- Policy: Hard Delete only (No soft delete). Protected by foreign key constraints.
-- =====================

CREATE TYPE theme_visibility AS ENUM ('public', 'private', 'restricted');

CREATE TABLE IF NOT EXISTS sys_themes (
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    name VARCHAR(100) NOT NULL,
    description TEXT,
    
    -- The styling config (colors, radius, fonts)
    config JSONB NOT NULL DEFAULT '{}',
    
    -- Ownership & Scope
    tenant_id UUID REFERENCES auth_tenants(id) ON DELETE CASCADE,
    
    -- Network Control Level
    visibility theme_visibility NOT NULL DEFAULT 'private',
    
    -- For Versioning/Updates
    version VARCHAR(20) DEFAULT '1.0.0',
    
    -- Metadata
    is_active BOOLEAN DEFAULT TRUE,
    created_at TIMESTAMPTZ DEFAULT NOW(),
    updated_at TIMESTAMPTZ DEFAULT NOW()
    
    -- NOTE: No deleted_at column. Themes use Hard Delete.
    -- Safety is enforced by auth_tenants(active_theme_id) FK constraint below.
    -- You cannot delete a theme if it is currently active on any tenant.
);

-- Access Control for Restricted Themes
CREATE TABLE IF NOT EXISTS sys_theme_access (
    theme_id UUID REFERENCES sys_themes(id) ON DELETE CASCADE,
    tenant_id UUID REFERENCES auth_tenants(id) ON DELETE CASCADE,
    granted_at TIMESTAMPTZ DEFAULT NOW(),
    granted_by UUID, 
    PRIMARY KEY (theme_id, tenant_id)
);

CREATE INDEX idx_themes_tenant ON sys_themes(tenant_id);
CREATE INDEX idx_themes_visibility ON sys_themes(visibility);

-- Update Tenant to have an 'active_theme'
-- Defaults to RESTRICT delete (Standard Postgres behavior).
-- If a theme is in use, it cannot be deleted.
ALTER TABLE auth_tenants 
ADD COLUMN IF NOT EXISTS active_theme_id UUID REFERENCES sys_themes(id);

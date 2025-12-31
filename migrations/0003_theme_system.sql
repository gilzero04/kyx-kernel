-- ================================================
-- Migration: 0003_theme_system.sql
-- Purpose: Theme registry and multi-tenant theme scoping.
-- ================================================

-- 1. Custom Types
DO $$ BEGIN
    CREATE TYPE theme_visibility AS ENUM ('public', 'private', 'restricted');
EXCEPTION
    WHEN duplicate_object THEN null;
END $$;

-- 2. Theme Registry
CREATE TABLE IF NOT EXISTS sys_themes (
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    tenant_id UUID REFERENCES auth_tenants(id) ON DELETE CASCADE,
    name VARCHAR(255) NOT NULL,
    description TEXT,
    config JSONB NOT NULL DEFAULT '{}'::jsonb,
    
    visibility theme_visibility NOT NULL DEFAULT 'private',
    version VARCHAR(20) DEFAULT '1.0.0',
    author VARCHAR(255),
    preview_url TEXT,
    logo_url TEXT,
    
    is_active BOOLEAN DEFAULT TRUE,
    is_system BOOLEAN DEFAULT FALSE,
    created_at TIMESTAMPTZ DEFAULT NOW(),
    updated_at TIMESTAMPTZ DEFAULT NOW(),
    UNIQUE(tenant_id, name)
);

CREATE TRIGGER update_sys_themes_updated_at BEFORE UPDATE ON sys_themes FOR EACH ROW EXECUTE FUNCTION update_updated_at_column();

-- 3. Theme Scoping for Tenants (Runtime Preferences)
-- These should use UUID to match sys_themes(id)
ALTER TABLE auth_tenants ADD COLUMN IF NOT EXISTS active_theme_id UUID REFERENCES sys_themes(id) ON DELETE SET NULL;
ALTER TABLE auth_tenants ADD COLUMN IF NOT EXISTS workspace_theme_light_id UUID REFERENCES sys_themes(id) ON DELETE SET NULL;
ALTER TABLE auth_tenants ADD COLUMN IF NOT EXISTS workspace_theme_dark_id UUID REFERENCES sys_themes(id) ON DELETE SET NULL;
ALTER TABLE auth_tenants ADD COLUMN IF NOT EXISTS app_theme_light_id UUID REFERENCES sys_themes(id) ON DELETE SET NULL;
ALTER TABLE auth_tenants ADD COLUMN IF NOT EXISTS app_theme_dark_id UUID REFERENCES sys_themes(id) ON DELETE SET NULL;

-- 4. User Preferences (Theme, Language per-user per-tenant)
CREATE TABLE IF NOT EXISTS user_preferences (
    user_id UUID NOT NULL REFERENCES auth_users(id) ON DELETE CASCADE,
    tenant_id UUID NOT NULL REFERENCES auth_tenants(id) ON DELETE CASCADE,
    theme_mode VARCHAR(20) DEFAULT 'system', -- 'light', 'dark', 'system'
    language VARCHAR(10) DEFAULT 'en',
    sidebar_collapsed BOOLEAN DEFAULT FALSE,
    settings JSONB DEFAULT '{}'::jsonb,
    updated_at TIMESTAMPTZ DEFAULT NOW(),
    PRIMARY KEY (user_id, tenant_id)
);

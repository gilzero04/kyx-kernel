-- ════════════════════════════════════════════════════════════════════════════
-- Migration: 0007_branding.sql
-- Purpose: Branding architecture with per-context themes
-- Consolidated from: 0024, 0025, 0027
-- ════════════════════════════════════════════════════════════════════════════

-- ════════════════════════════════════════════════════════════════════════════
-- 1. BRANDING TABLE (Single Source of Truth)
-- ════════════════════════════════════════════════════════════════════════════

CREATE TABLE IF NOT EXISTS sys_brandings (
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    tenant_id UUID REFERENCES auth_tenants(id) ON DELETE CASCADE,
    context VARCHAR(20) DEFAULT 'workspace' CHECK (context IN ('console', 'workspace')),
    
    -- Identity
    name VARCHAR(255) NOT NULL,
    description TEXT,
    
    -- Visual Assets
    logo_light_url TEXT,
    logo_dark_url TEXT,
    favicon_url TEXT,
    icon_app_url TEXT,
    splash_image_url TEXT,
    
    -- Colors
    primary_color VARCHAR(20),
    secondary_color VARCHAR(20),
    accent_color VARCHAR(20),
    
    -- Text/Display
    app_name VARCHAR(255),
    splash_text VARCHAR(255),
    splash_subtext VARCHAR(255),
    tagline VARCHAR(255),
    
    -- Console/Default Theme References
    theme_light_id UUID REFERENCES sys_themes(id) ON DELETE SET NULL,
    theme_dark_id UUID REFERENCES sys_themes(id) ON DELETE SET NULL,
    
    -- Per-Context Theme Overrides
    theme_workspace_light_id UUID REFERENCES sys_themes(id) ON DELETE SET NULL,
    theme_workspace_dark_id UUID REFERENCES sys_themes(id) ON DELETE SET NULL,
    theme_app_light_id UUID REFERENCES sys_themes(id) ON DELETE SET NULL,
    theme_app_dark_id UUID REFERENCES sys_themes(id) ON DELETE SET NULL,
    
    -- Extensible Metadata
    metadata JSONB DEFAULT '{}',
    
    -- Timestamps
    created_at TIMESTAMPTZ DEFAULT NOW(),
    updated_at TIMESTAMPTZ DEFAULT NOW()
);

-- Indexes
CREATE INDEX IF NOT EXISTS idx_sys_brandings_name ON sys_brandings(name);
CREATE UNIQUE INDEX IF NOT EXISTS idx_sys_brandings_tenant_context 
    ON sys_brandings(tenant_id, context) WHERE tenant_id IS NOT NULL;

-- UpdatedAt Trigger
CREATE OR REPLACE FUNCTION update_sys_brandings_updated_at()
RETURNS TRIGGER AS $$
BEGIN
    NEW.updated_at = NOW();
    RETURN NEW;
END;
$$ language 'plpgsql';

CREATE TRIGGER sys_brandings_updated_at_trigger
    BEFORE UPDATE ON sys_brandings
    FOR EACH ROW EXECUTE FUNCTION update_sys_brandings_updated_at();

-- ════════════════════════════════════════════════════════════════════════════
-- 2. ADD BRANDING FK TO TENANTS
-- ════════════════════════════════════════════════════════════════════════════

ALTER TABLE auth_tenants 
    ADD CONSTRAINT fk_auth_tenants_branding 
    FOREIGN KEY (branding_id) REFERENCES sys_brandings(id) ON DELETE SET NULL;

-- ════════════════════════════════════════════════════════════════════════════
-- 3. WORKSPACE CONTEXT TABLE
-- ════════════════════════════════════════════════════════════════════════════

CREATE TABLE IF NOT EXISTS sys_workspace_contexts (
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    user_id UUID NOT NULL REFERENCES auth_users(id) ON DELETE CASCADE,
    session_id VARCHAR(255),
    active_tenant_id UUID NOT NULL REFERENCES auth_tenants(id) ON DELETE CASCADE,
    ui_overrides JSONB DEFAULT '{}',
    created_at TIMESTAMPTZ DEFAULT NOW(),
    updated_at TIMESTAMPTZ DEFAULT NOW(),
    CONSTRAINT unique_user_session UNIQUE (user_id, session_id)
);

CREATE INDEX IF NOT EXISTS idx_workspace_contexts_user ON sys_workspace_contexts(user_id);
CREATE INDEX IF NOT EXISTS idx_workspace_contexts_tenant ON sys_workspace_contexts(active_tenant_id);

CREATE TRIGGER sys_workspace_contexts_updated_at
    BEFORE UPDATE ON sys_workspace_contexts
    FOR EACH ROW EXECUTE FUNCTION update_updated_at_column();

-- ════════════════════════════════════════════════════════════════════════════
-- COMMENTS
-- ════════════════════════════════════════════════════════════════════════════

COMMENT ON TABLE sys_brandings IS 'Single source of truth for branding configurations';
COMMENT ON COLUMN sys_brandings.context IS 'Branding context: console or workspace';
COMMENT ON COLUMN sys_brandings.tenant_id IS 'Direct reference to owning tenant';
COMMENT ON COLUMN sys_brandings.theme_light_id IS 'Default/Console theme for light mode';
COMMENT ON COLUMN sys_brandings.theme_dark_id IS 'Default/Console theme for dark mode';
COMMENT ON COLUMN sys_brandings.theme_workspace_light_id IS 'Workspace theme override for light mode';
COMMENT ON COLUMN sys_brandings.theme_workspace_dark_id IS 'Workspace theme override for dark mode';
COMMENT ON COLUMN sys_brandings.theme_app_light_id IS 'App theme override for light mode';
COMMENT ON COLUMN sys_brandings.theme_app_dark_id IS 'App theme override for dark mode';
COMMENT ON TABLE sys_workspace_contexts IS 'Tracks active tenant context per user session';

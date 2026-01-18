-- ════════════════════════════════════════════════════════════════════════════
-- Migration: 0008_sharing.sql (CONSOLIDATED)
-- Purpose: Resource sharing system for roles, themes, pages, media, i18n
-- Features: Broadcast sharing (is_shared), Explicit sharing, Cascading
-- Note: This is a consolidated version merging 0008, 0009, 0010, 0011
-- ════════════════════════════════════════════════════════════════════════════

-- ════════════════════════════════════════════════════════════════════════════
-- 1. ADD is_shared COLUMN TO SHAREABLE RESOURCES
-- When is_shared = TRUE, all descendants can see the resource
-- ════════════════════════════════════════════════════════════════════════════

ALTER TABLE sys_roles ADD COLUMN IF NOT EXISTS is_shared BOOLEAN DEFAULT FALSE;
ALTER TABLE sys_themes ADD COLUMN IF NOT EXISTS is_shared BOOLEAN DEFAULT FALSE;
ALTER TABLE sys_pages ADD COLUMN IF NOT EXISTS is_shared BOOLEAN DEFAULT FALSE;
ALTER TABLE media_assets ADD COLUMN IF NOT EXISTS is_shared BOOLEAN DEFAULT FALSE;
ALTER TABLE sys_i18n_translations ADD COLUMN IF NOT EXISTS is_shared BOOLEAN DEFAULT FALSE;

COMMENT ON COLUMN sys_roles.is_shared IS 'If TRUE, all descendant tenants can see this role';
COMMENT ON COLUMN sys_themes.is_shared IS 'If TRUE, all descendant tenants can use this theme';
COMMENT ON COLUMN sys_pages.is_shared IS 'If TRUE, all descendant tenants can view this page template';
COMMENT ON COLUMN media_assets.is_shared IS 'If TRUE, all descendant tenants can use this media asset';
COMMENT ON COLUMN sys_i18n_translations.is_shared IS 'If TRUE, all descendant tenants can use these translations';

-- ════════════════════════════════════════════════════════════════════════════
-- 2. DROP OLD visibility COLUMN (replaced by is_shared)
-- ════════════════════════════════════════════════════════════════════════════

-- Migrate visibility data to is_shared first (only if column exists)
DO $$
BEGIN
    IF EXISTS (SELECT 1 FROM information_schema.columns 
               WHERE table_name = 'sys_themes' AND column_name = 'visibility') THEN
        UPDATE sys_themes SET is_shared = TRUE WHERE visibility = 'public';
        ALTER TABLE sys_themes DROP COLUMN visibility;
    END IF;
END $$;
DROP TABLE IF EXISTS sys_theme_access;

-- ════════════════════════════════════════════════════════════════════════════
-- 3. EXPLICIT SHARING TABLE
-- For sharing specific resources to specific tenants
-- ════════════════════════════════════════════════════════════════════════════

CREATE TABLE IF NOT EXISTS sys_resource_shares (
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    resource_type VARCHAR(30) NOT NULL CHECK (resource_type IN ('role', 'theme', 'page', 'media', 'permission', 'i18n')),
    resource_id UUID NOT NULL,
    owner_tenant_id UUID NOT NULL REFERENCES auth_tenants(id) ON DELETE CASCADE,
    shared_to_tenant_id UUID NOT NULL REFERENCES auth_tenants(id) ON DELETE CASCADE,
    can_reshare BOOLEAN DEFAULT FALSE,
    is_active BOOLEAN DEFAULT TRUE,
    created_by UUID REFERENCES auth_users(id),
    created_at TIMESTAMPTZ DEFAULT NOW(),
    updated_at TIMESTAMPTZ DEFAULT NOW(),
    UNIQUE(resource_type, resource_id, shared_to_tenant_id)
);

CREATE INDEX IF NOT EXISTS idx_resource_shares_to_tenant ON sys_resource_shares(shared_to_tenant_id) WHERE is_active = TRUE;
CREATE INDEX IF NOT EXISTS idx_resource_shares_resource ON sys_resource_shares(resource_type, resource_id) WHERE is_active = TRUE;
CREATE INDEX IF NOT EXISTS idx_resource_shares_owner ON sys_resource_shares(owner_tenant_id) WHERE is_active = TRUE;

-- Trigger (use OR REPLACE via DROP first)
DROP TRIGGER IF EXISTS update_sys_resource_shares_updated_at ON sys_resource_shares;
CREATE TRIGGER update_sys_resource_shares_updated_at 
    BEFORE UPDATE ON sys_resource_shares 
    FOR EACH ROW EXECUTE FUNCTION update_updated_at_column();

-- ════════════════════════════════════════════════════════════════════════════
-- 4. HELPER FUNCTION: Check if tenant can access shared resource
-- IMPORTANT: Drop ALL versions first to avoid "function name is not unique"
-- ════════════════════════════════════════════════════════════════════════════

DROP FUNCTION IF EXISTS can_access_shared_resource(VARCHAR, UUID, UUID);
DROP FUNCTION IF EXISTS can_access_shared_resource(VARCHAR(30), UUID, UUID);
DROP FUNCTION IF EXISTS can_access_shared_resource(TEXT, UUID, UUID);

CREATE FUNCTION can_access_shared_resource(
    p_resource_type TEXT,
    p_resource_id UUID,
    p_viewer_tenant_id UUID
) RETURNS BOOLEAN AS $$
DECLARE
    v_owner_tenant_id UUID;
    v_is_shared BOOLEAN;
BEGIN
    -- Get resource owner and is_shared status based on resource type
    -- NOTE: sys_themes, media_assets do NOT have deleted_at column
    CASE p_resource_type
        WHEN 'role' THEN
            SELECT tenant_id, is_shared INTO v_owner_tenant_id, v_is_shared
            FROM sys_roles WHERE id = p_resource_id AND deleted_at IS NULL;
        WHEN 'theme' THEN
            SELECT tenant_id, is_shared INTO v_owner_tenant_id, v_is_shared
            FROM sys_themes WHERE id = p_resource_id;
        WHEN 'page' THEN
            SELECT tenant_id, is_shared INTO v_owner_tenant_id, v_is_shared
            FROM sys_pages WHERE id = p_resource_id AND deleted_at IS NULL;
        WHEN 'media' THEN
            SELECT tenant_id, is_shared INTO v_owner_tenant_id, v_is_shared
            FROM media_assets WHERE id = p_resource_id;
        WHEN 'i18n' THEN
            SELECT tenant_id, is_shared INTO v_owner_tenant_id, v_is_shared
            FROM sys_i18n_translations WHERE id = p_resource_id;
        ELSE
            RETURN FALSE;
    END CASE;
    
    -- Check 1: Viewer is owner
    IF v_owner_tenant_id = p_viewer_tenant_id THEN
        RETURN TRUE;
    END IF;
    
    -- Check 2: Broadcast sharing (is_shared = TRUE and viewer is descendant)
    IF v_is_shared = TRUE AND can_view_tenant(v_owner_tenant_id, p_viewer_tenant_id) THEN
        RETURN TRUE;
    END IF;
    
    -- Check 3: Explicit sharing via sys_resource_shares
    IF EXISTS (
        SELECT 1 FROM sys_resource_shares
        WHERE resource_type = p_resource_type
          AND resource_id = p_resource_id
          AND shared_to_tenant_id = p_viewer_tenant_id
          AND is_active = TRUE
    ) THEN
        RETURN TRUE;
    END IF;
    
    -- Check 4: Cascaded sharing (ancestor has can_reshare)
    IF EXISTS (
        SELECT 1 FROM sys_resource_shares srs
        WHERE srs.resource_type = p_resource_type
          AND srs.resource_id = p_resource_id
          AND srs.can_reshare = TRUE
          AND srs.is_active = TRUE
          AND can_view_tenant(srs.shared_to_tenant_id, p_viewer_tenant_id)
    ) THEN
        RETURN TRUE;
    END IF;
    
    RETURN FALSE;
END;
$$ LANGUAGE plpgsql STABLE;

-- ════════════════════════════════════════════════════════════════════════════
-- 5. HELPER FUNCTION: Check if resource is in use before revoke
-- ════════════════════════════════════════════════════════════════════════════

DROP FUNCTION IF EXISTS get_resource_usage_count(VARCHAR, UUID, UUID);
DROP FUNCTION IF EXISTS get_resource_usage_count(VARCHAR(30), UUID, UUID);
DROP FUNCTION IF EXISTS get_resource_usage_count(TEXT, UUID, UUID);

CREATE FUNCTION get_resource_usage_count(
    p_resource_type TEXT,
    p_resource_id UUID,
    p_tenant_id UUID
) RETURNS INTEGER AS $$
DECLARE
    v_count INTEGER := 0;
BEGIN
    CASE p_resource_type
        WHEN 'role' THEN
            SELECT COUNT(*) INTO v_count
            FROM auth_memberships 
            WHERE role_id = p_resource_id 
              AND tenant_id = p_tenant_id 
              AND deleted_at IS NULL;
        WHEN 'theme' THEN
            SELECT COUNT(*) INTO v_count
            FROM sys_brandings 
            WHERE tenant_id = p_tenant_id 
              AND (theme_light_id = p_resource_id 
                   OR theme_dark_id = p_resource_id
                   OR theme_workspace_light_id = p_resource_id
                   OR theme_workspace_dark_id = p_resource_id
                   OR theme_app_light_id = p_resource_id
                   OR theme_app_dark_id = p_resource_id);
        ELSE
            v_count := 0;
    END CASE;
    
    RETURN v_count;
END;
$$ LANGUAGE plpgsql STABLE;

-- ════════════════════════════════════════════════════════════════════════════
-- 6. CONSTRAINT: Shares must be within same network
-- ════════════════════════════════════════════════════════════════════════════

DROP FUNCTION IF EXISTS check_share_network() CASCADE;

CREATE FUNCTION check_share_network() RETURNS TRIGGER AS $$
BEGIN
    IF NOT (can_view_tenant(NEW.owner_tenant_id, NEW.shared_to_tenant_id) 
         OR can_view_tenant(NEW.shared_to_tenant_id, NEW.owner_tenant_id)) THEN
        RAISE EXCEPTION 'Cannot share resources across different networks';
    END IF;
    RETURN NEW;
END;
$$ LANGUAGE plpgsql;

DROP TRIGGER IF EXISTS trg_check_share_network ON sys_resource_shares;
CREATE TRIGGER trg_check_share_network
    BEFORE INSERT OR UPDATE ON sys_resource_shares
    FOR EACH ROW EXECUTE FUNCTION check_share_network();

-- ════════════════════════════════════════════════════════════════════════════
-- COMMENTS
-- ════════════════════════════════════════════════════════════════════════════

COMMENT ON TABLE sys_resource_shares IS 'Explicit resource sharing between tenants';
COMMENT ON COLUMN sys_resource_shares.can_reshare IS 'If TRUE, recipient can share to their children (cascading)';
COMMENT ON FUNCTION can_access_shared_resource IS 'Check if tenant can access a shared resource (broadcast or explicit). Supports: role, theme, page, media, i18n.';
COMMENT ON FUNCTION get_resource_usage_count IS 'Check usage count before revoking share';

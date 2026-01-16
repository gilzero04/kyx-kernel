-- ════════════════════════════════════════════════════════════════════════════
-- Migration: 0008_sharing.sql
-- Purpose: Resource sharing system for roles, themes, pages, media
-- Features: Broadcast sharing (is_shared), Explicit sharing, Cascading
-- ════════════════════════════════════════════════════════════════════════════

-- ════════════════════════════════════════════════════════════════════════════
-- 1. ADD is_shared COLUMN TO SHAREABLE RESOURCES
-- When is_shared = TRUE, all descendants can see the resource
-- ════════════════════════════════════════════════════════════════════════════

ALTER TABLE sys_roles ADD COLUMN IF NOT EXISTS is_shared BOOLEAN DEFAULT FALSE;
ALTER TABLE sys_themes ADD COLUMN IF NOT EXISTS is_shared BOOLEAN DEFAULT FALSE;
ALTER TABLE sys_pages ADD COLUMN IF NOT EXISTS is_shared BOOLEAN DEFAULT FALSE;
ALTER TABLE media_assets ADD COLUMN IF NOT EXISTS is_shared BOOLEAN DEFAULT FALSE;

COMMENT ON COLUMN sys_roles.is_shared IS 'If TRUE, all descendant tenants can see this role';
COMMENT ON COLUMN sys_themes.is_shared IS 'If TRUE, all descendant tenants can use this theme';
COMMENT ON COLUMN sys_pages.is_shared IS 'If TRUE, all descendant tenants can view this page template';
COMMENT ON COLUMN media_assets.is_shared IS 'If TRUE, all descendant tenants can use this media asset';

-- ════════════════════════════════════════════════════════════════════════════
-- 2. EXPLICIT SHARING TABLE
-- For sharing specific resources to specific tenants
-- ════════════════════════════════════════════════════════════════════════════

CREATE TABLE IF NOT EXISTS sys_resource_shares (
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    resource_type VARCHAR(30) NOT NULL CHECK (resource_type IN ('role', 'theme', 'page', 'media', 'permission')),
    resource_id UUID NOT NULL,
    owner_tenant_id UUID NOT NULL REFERENCES auth_tenants(id) ON DELETE CASCADE,
    shared_to_tenant_id UUID NOT NULL REFERENCES auth_tenants(id) ON DELETE CASCADE,
    can_reshare BOOLEAN DEFAULT FALSE, -- Allow cascading shares
    is_active BOOLEAN DEFAULT TRUE,
    created_by UUID REFERENCES auth_users(id),
    created_at TIMESTAMPTZ DEFAULT NOW(),
    updated_at TIMESTAMPTZ DEFAULT NOW(),
    
    -- Prevent duplicate shares
    UNIQUE(resource_type, resource_id, shared_to_tenant_id)
);

-- Indexes for fast lookups
CREATE INDEX IF NOT EXISTS idx_resource_shares_to_tenant ON sys_resource_shares(shared_to_tenant_id) WHERE is_active = TRUE;
CREATE INDEX IF NOT EXISTS idx_resource_shares_resource ON sys_resource_shares(resource_type, resource_id) WHERE is_active = TRUE;
CREATE INDEX IF NOT EXISTS idx_resource_shares_owner ON sys_resource_shares(owner_tenant_id) WHERE is_active = TRUE;

-- Trigger for updated_at
CREATE TRIGGER update_sys_resource_shares_updated_at 
    BEFORE UPDATE ON sys_resource_shares 
    FOR EACH ROW EXECUTE FUNCTION update_updated_at_column();

-- ════════════════════════════════════════════════════════════════════════════
-- 3. HELPER FUNCTION: Check if tenant can access shared resource
-- ════════════════════════════════════════════════════════════════════════════

CREATE OR REPLACE FUNCTION can_access_shared_resource(
    p_resource_type VARCHAR(30),
    p_resource_id UUID,
    p_viewer_tenant_id UUID
) RETURNS BOOLEAN AS $$
DECLARE
    v_owner_tenant_id UUID;
    v_is_shared BOOLEAN;
BEGIN
    -- Get resource owner and is_shared status based on resource type
    CASE p_resource_type
        WHEN 'role' THEN
            SELECT tenant_id, is_shared INTO v_owner_tenant_id, v_is_shared
            FROM sys_roles WHERE id = p_resource_id AND deleted_at IS NULL;
        WHEN 'theme' THEN
            SELECT tenant_id, is_shared INTO v_owner_tenant_id, v_is_shared
            FROM sys_themes WHERE id = p_resource_id AND deleted_at IS NULL;
        WHEN 'page' THEN
            SELECT tenant_id, is_shared INTO v_owner_tenant_id, v_is_shared
            FROM sys_pages WHERE id = p_resource_id AND deleted_at IS NULL;
        WHEN 'media' THEN
            SELECT tenant_id, is_shared INTO v_owner_tenant_id, v_is_shared
            FROM media_assets WHERE id = p_resource_id AND deleted_at IS NULL;
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
    
    -- Check 3: Explicit sharing (direct or cascaded)
    RETURN EXISTS (
        WITH RECURSIVE share_chain AS (
            -- Direct shares to viewer
            SELECT resource_id, owner_tenant_id, can_reshare, 1 as depth
            FROM sys_resource_shares 
            WHERE resource_type = p_resource_type 
              AND resource_id = p_resource_id 
              AND shared_to_tenant_id = p_viewer_tenant_id
              AND is_active = TRUE
            UNION ALL
            -- Cascaded shares (if can_reshare = TRUE)
            SELECT s.resource_id, s.owner_tenant_id, s.can_reshare, sc.depth + 1
            FROM sys_resource_shares s
            JOIN share_chain sc ON s.resource_id = sc.resource_id
            WHERE s.shared_to_tenant_id = p_viewer_tenant_id
              AND s.is_active = TRUE
              AND sc.can_reshare = TRUE
              AND sc.depth < 10  -- Prevent infinite recursion
        )
        SELECT 1 FROM share_chain LIMIT 1
    );
END;
$$ LANGUAGE plpgsql STABLE;

-- ════════════════════════════════════════════════════════════════════════════
-- 4. HELPER FUNCTION: Check if resource is in use before revoke
-- ════════════════════════════════════════════════════════════════════════════

CREATE OR REPLACE FUNCTION get_resource_usage_count(
    p_resource_type VARCHAR(30),
    p_resource_id UUID,
    p_tenant_id UUID  -- The tenant we want to revoke from
) RETURNS INTEGER AS $$
DECLARE
    v_count INTEGER := 0;
BEGIN
    CASE p_resource_type
        WHEN 'role' THEN
            -- Count memberships using this role in the target tenant
            SELECT COUNT(*) INTO v_count
            FROM auth_memberships 
            WHERE role_id = p_resource_id 
              AND tenant_id = p_tenant_id 
              AND deleted_at IS NULL;
        WHEN 'theme' THEN
            -- Count brandings using this theme in the target tenant
            SELECT COUNT(*) INTO v_count
            FROM sys_brandings 
            WHERE tenant_id = p_tenant_id 
              AND (theme_light_id = p_resource_id 
                   OR theme_dark_id = p_resource_id
                   OR theme_workspace_light_id = p_resource_id
                   OR theme_workspace_dark_id = p_resource_id
                   OR theme_app_light_id = p_resource_id
                   OR theme_app_dark_id = p_resource_id);
        WHEN 'page' THEN
            -- Pages typically don't have usage count
            v_count := 0;
        WHEN 'media' THEN
            -- Could check if media is used in CMS pages, but complex
            v_count := 0;
        ELSE
            v_count := 0;
    END CASE;
    
    RETURN v_count;
END;
$$ LANGUAGE plpgsql STABLE;

-- ════════════════════════════════════════════════════════════════════════════
-- 5. CONSTRAINT: Shares must be within same network
-- ════════════════════════════════════════════════════════════════════════════

CREATE OR REPLACE FUNCTION check_share_network() RETURNS TRIGGER AS $$
DECLARE
    v_owner_root UUID;
    v_target_root UUID;
BEGIN
    -- Get root of owner
    SELECT id INTO v_owner_root FROM auth_tenants 
    WHERE id = (SELECT ancestor FROM unnest(get_ancestor_chain(NEW.owner_tenant_id)) AS ancestor ORDER BY 1 LIMIT 1);
    
    -- Get root of target
    SELECT id INTO v_target_root FROM auth_tenants 
    WHERE id = (SELECT ancestor FROM unnest(get_ancestor_chain(NEW.shared_to_tenant_id)) AS ancestor ORDER BY 1 LIMIT 1);
    
    -- If roots are different, deny (unless one is platform owner which has no parent)
    -- For simplicity, we check if owner can view target or target can view owner
    IF NOT (can_view_tenant(NEW.owner_tenant_id, NEW.shared_to_tenant_id) 
         OR can_view_tenant(NEW.shared_to_tenant_id, NEW.owner_tenant_id)) THEN
        RAISE EXCEPTION 'Cannot share resources across different networks';
    END IF;
    
    RETURN NEW;
END;
$$ LANGUAGE plpgsql;

CREATE TRIGGER trg_check_share_network
    BEFORE INSERT OR UPDATE ON sys_resource_shares
    FOR EACH ROW EXECUTE FUNCTION check_share_network();

-- ════════════════════════════════════════════════════════════════════════════
-- COMMENTS
-- ════════════════════════════════════════════════════════════════════════════

COMMENT ON TABLE sys_resource_shares IS 'Explicit resource sharing between tenants';
COMMENT ON COLUMN sys_resource_shares.can_reshare IS 'If TRUE, recipient can share to their children (cascading)';
COMMENT ON FUNCTION can_access_shared_resource IS 'Check if tenant can access a shared resource (broadcast or explicit)';
COMMENT ON FUNCTION get_resource_usage_count IS 'Check usage count before revoking share';

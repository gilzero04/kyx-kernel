-- ════════════════════════════════════════════════════════════════════════════
-- Migration: 0011_fix_theme_deleted_at.sql
-- Purpose: Fix can_access_shared_resource function - sys_themes has no deleted_at
-- ════════════════════════════════════════════════════════════════════════════

-- Drop first to avoid "function name is not unique" error
DROP FUNCTION IF EXISTS can_access_shared_resource(VARCHAR(30), UUID, UUID);

-- Update function to remove deleted_at check for sys_themes and media_assets
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
    -- NOTE: Only sys_roles and sys_pages have deleted_at column
    CASE p_resource_type
        WHEN 'role' THEN
            SELECT tenant_id, is_shared INTO v_owner_tenant_id, v_is_shared
            FROM sys_roles WHERE id = p_resource_id AND deleted_at IS NULL;
        WHEN 'theme' THEN
            -- sys_themes has NO deleted_at column
            SELECT tenant_id, is_shared INTO v_owner_tenant_id, v_is_shared
            FROM sys_themes WHERE id = p_resource_id;
        WHEN 'page' THEN
            SELECT tenant_id, is_shared INTO v_owner_tenant_id, v_is_shared
            FROM sys_pages WHERE id = p_resource_id AND deleted_at IS NULL;
        WHEN 'media' THEN
            -- media_assets has NO deleted_at column
            SELECT tenant_id, is_shared INTO v_owner_tenant_id, v_is_shared
            FROM media_assets WHERE id = p_resource_id;
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

COMMENT ON FUNCTION can_access_shared_resource IS 'Check if tenant can access a shared resource (broadcast or explicit). Note: sys_themes and media_assets do not have deleted_at column.';

-- ════════════════════════════════════════════════════════════════════════════
-- Migration: 0010_i18n_sharing.sql
-- Purpose: Add is_shared column to sys_i18n_translations for translation sharing
-- ════════════════════════════════════════════════════════════════════════════

-- 1. Add is_shared column to i18n translations
ALTER TABLE sys_i18n_translations ADD COLUMN IF NOT EXISTS is_shared BOOLEAN DEFAULT FALSE;

COMMENT ON COLUMN sys_i18n_translations.is_shared IS 'If TRUE, all descendant tenants can use these translations';

-- 2. Update can_access_shared_resource function to include i18n
CREATE OR REPLACE FUNCTION can_access_shared_resource(
    p_resource_type TEXT,
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
            FROM sys_roles WHERE id = p_resource_id;
        WHEN 'theme' THEN
            SELECT tenant_id, is_shared INTO v_owner_tenant_id, v_is_shared
            FROM sys_themes WHERE id = p_resource_id;
        WHEN 'page' THEN
            SELECT tenant_id, is_shared INTO v_owner_tenant_id, v_is_shared
            FROM sys_pages WHERE id = p_resource_id;
        WHEN 'media' THEN
            SELECT tenant_id, is_shared INTO v_owner_tenant_id, v_is_shared
            FROM media_assets WHERE id = p_resource_id;
        WHEN 'i18n' THEN
            SELECT tenant_id, is_shared INTO v_owner_tenant_id, v_is_shared
            FROM sys_i18n_translations WHERE id = p_resource_id;
        ELSE
            RETURN FALSE;
    END CASE;
    
    -- Check 1: Viewer owns the resource
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
        JOIN auth_tenants at ON at.id = p_viewer_tenant_id
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

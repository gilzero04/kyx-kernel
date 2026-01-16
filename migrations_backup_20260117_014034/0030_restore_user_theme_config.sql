-- Migration: 0030_restore_user_theme_config.sql
-- Purpose: Restore user's intentional theme configuration
-- 
-- Context: Migration 0029 incorrectly "fixed" the theme assignments.
-- The user INTENTIONALLY configured:
--   - theme_light_id = kyx-dark (user wants dark theme when Light Mode is selected)
--   - theme_dark_id = kyx-light (user wants light theme when Dark Mode is selected)
--
-- This is NOT a bug - themes can be assigned to any mode!
-- The mode toggle (Light/Dark) determines the experience, not the theme name.

DO $$
DECLARE
    light_theme_id UUID;
    dark_theme_id UUID;
BEGIN
    -- Get theme IDs by slug
    SELECT id INTO light_theme_id FROM sys_themes WHERE slug = 'kyx-light' LIMIT 1;
    SELECT id INTO dark_theme_id FROM sys_themes WHERE slug = 'kyx-dark' LIMIT 1;
    
    IF light_theme_id IS NOT NULL AND dark_theme_id IS NOT NULL THEN
        -- Restore user's INTENTIONAL configuration:
        -- Light Mode uses dark theme, Dark Mode uses light theme
        UPDATE sys_brandings
        SET 
            theme_light_id = dark_theme_id,  -- kyx-dark for Light Mode
            theme_dark_id = light_theme_id,  -- kyx-light for Dark Mode
            theme_workspace_light_id = dark_theme_id,
            theme_workspace_dark_id = light_theme_id,
            theme_app_light_id = dark_theme_id,
            theme_app_dark_id = light_theme_id,
            updated_at = NOW()
        WHERE 
            -- Only revert records that were incorrectly "fixed" by 0029
            theme_light_id = light_theme_id AND theme_dark_id = dark_theme_id;
            
        RAISE NOTICE 'Restored user theme configuration';
    ELSE
        RAISE WARNING 'Could not find themes. Migration skipped.';
    END IF;
END $$;

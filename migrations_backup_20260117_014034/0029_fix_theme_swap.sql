-- Migration: 0029_fix_theme_swap.sql
-- NOTE: This migration was incorrectly applied and is now superseded by 0030.
-- Keeping this file as placeholder since sqlx tracks applied migrations.
-- See 0030_restore_user_theme_config.sql for the correct configuration.

-- Original logic was incorrect - it assumed theme names must match mode names.
-- CORRECT PRINCIPLE: Any theme can be assigned to any mode!
-- The mode toggle in header determines which theme to use, not the theme name.

-- No changes in this migration - all fixes are in 0030
SELECT 1;

-- ════════════════════════════════════════════════════════════════════════════
-- Migration: 0009_consolidate_sharing.sql
-- Purpose: Remove redundant visibility column, use is_shared only
-- ════════════════════════════════════════════════════════════════════════════

-- 1. Migrate visibility data to is_shared
-- public → is_shared = TRUE
-- private/restricted → is_shared = FALSE (default)
UPDATE sys_themes SET is_shared = TRUE WHERE visibility = 'public';

-- 2. Drop visibility column (no longer needed)
ALTER TABLE sys_themes DROP COLUMN IF EXISTS visibility;

-- 3. Also drop sys_theme_access if exists (replaced by sys_resource_shares)
DROP TABLE IF EXISTS sys_theme_access;

-- 4. Add comment for clarity
COMMENT ON COLUMN sys_themes.is_shared IS 'If TRUE, all descendant tenants can see and use this theme. Replaces old visibility column.';

-- ════════════════════════════════════════════════════════════════════════════
-- Sharing Rules Summary (consolidated):
-- 
-- is_shared = TRUE → Broadcast to all descendants
-- is_shared = FALSE (default) → Private to owner OR explicit share via sys_resource_shares
-- ════════════════════════════════════════════════════════════════════════════

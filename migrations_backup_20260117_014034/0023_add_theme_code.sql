-- ================================================
-- Migration: 0023_add_theme_code.sql
-- Purpose: Add 'code' column to sys_themes for portable identification.
--          The 'code' comes from manifest.json 'id' field and is used
--          for matching themes across local development and production.
-- ================================================

-- 1. Add code column
ALTER TABLE sys_themes ADD COLUMN IF NOT EXISTS code VARCHAR(100);

-- 2. Backfill existing themes with code derived from name
-- Converts "Kyx Dark" -> "kyx-dark", "Kyx Light" -> "kyx-light"
UPDATE sys_themes 
SET code = LOWER(REPLACE(name, ' ', '-'))
WHERE code IS NULL;

-- 3. Make code unique (after backfill)
CREATE UNIQUE INDEX IF NOT EXISTS idx_sys_themes_code ON sys_themes(code);

-- 4. Add comment for documentation
COMMENT ON COLUMN sys_themes.code IS 'Human-readable identifier from manifest.json, used for portable theme matching';

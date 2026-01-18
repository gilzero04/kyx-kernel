-- ════════════════════════════════════════════════════════════════════════════
-- Migration: 0010_fix_api_key_prefix.sql
-- Purpose: Extend prefix column to accommodate full prefix format (kyx_sk_live_)
-- ════════════════════════════════════════════════════════════════════════════

-- Fix: prefix column was VARCHAR(10) but code generates 12-char prefix like "kyx_sk_live_"
ALTER TABLE sys_api_keys ALTER COLUMN prefix TYPE VARCHAR(15);

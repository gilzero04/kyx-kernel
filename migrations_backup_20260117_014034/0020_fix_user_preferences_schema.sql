-- ================================================
-- Migration: 0020_fix_user_preferences_schema.sql
-- Purpose: Align user_preferences table with repository expectations
-- ================================================

-- Drop and recreate user_preferences table with correct schema
DROP TABLE IF EXISTS user_preferences CASCADE;

CREATE TABLE user_preferences (
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    user_id UUID NOT NULL REFERENCES auth_users(id) ON DELETE CASCADE,
    preferred_theme_light_id VARCHAR(100),
    preferred_theme_dark_id VARCHAR(100),
    theme_mode VARCHAR(20) DEFAULT 'system',
    locale VARCHAR(10) DEFAULT 'en',
    timezone VARCHAR(50) DEFAULT 'UTC',
    notifications_enabled BOOLEAN DEFAULT TRUE,
    created_at TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    updated_at TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    CONSTRAINT uq_user_preferences_user UNIQUE(user_id)
);

-- Index for lookups
CREATE INDEX idx_user_preferences_user_id ON user_preferences(user_id);

COMMENT ON TABLE user_preferences IS 'User-specific preferences (theme, locale, notifications)';

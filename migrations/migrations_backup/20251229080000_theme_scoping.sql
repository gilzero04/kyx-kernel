-- Theme Scoping Architecture Migration
-- Adds workspace/app theme columns to auth_tenants and creates user_preferences table

-- 1. Add theme columns to auth_tenants
ALTER TABLE auth_tenants 
ADD COLUMN IF NOT EXISTS workspace_theme_light_id VARCHAR(100) DEFAULT 'light',
ADD COLUMN IF NOT EXISTS workspace_theme_dark_id VARCHAR(100) DEFAULT 'dark',
ADD COLUMN IF NOT EXISTS app_theme_light_id VARCHAR(100) DEFAULT 'light',
ADD COLUMN IF NOT EXISTS app_theme_dark_id VARCHAR(100) DEFAULT 'dark';

-- 2. Create user_preferences table for per-user theme preferences
CREATE TABLE IF NOT EXISTS user_preferences (
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    user_id UUID NOT NULL REFERENCES auth_users(id) ON DELETE CASCADE,
    preferred_theme_light_id VARCHAR(100),
    preferred_theme_dark_id VARCHAR(100),
    theme_mode VARCHAR(20) DEFAULT 'system', -- 'light', 'dark', 'system'
    locale VARCHAR(10),
    timezone VARCHAR(100),
    notifications_enabled BOOLEAN DEFAULT true,
    created_at TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    updated_at TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    UNIQUE(user_id)
);

-- 3. Create index for quick user lookup
CREATE INDEX IF NOT EXISTS idx_user_preferences_user_id ON user_preferences(user_id);

-- 4. Add trigger for updated_at
CREATE OR REPLACE FUNCTION update_user_preferences_updated_at()
RETURNS TRIGGER AS $$
BEGIN
    NEW.updated_at = NOW();
    RETURN NEW;
END;
$$ LANGUAGE plpgsql;

DROP TRIGGER IF EXISTS trg_user_preferences_updated_at ON user_preferences;
CREATE TRIGGER trg_user_preferences_updated_at
    BEFORE UPDATE ON user_preferences
    FOR EACH ROW
    EXECUTE FUNCTION update_user_preferences_updated_at();

-- 5. Rename sys_configs theme keys for clarity (platform console themes)
-- Existing: theme_light_id, theme_dark_id
-- These will now explicitly be for platform console
UPDATE sys_configs SET key = 'console_theme_light_id' WHERE key = 'theme_light_id';
UPDATE sys_configs SET key = 'console_theme_dark_id' WHERE key = 'theme_dark_id';

-- 6. Insert default console themes if not exists
INSERT INTO sys_configs (key, value)
VALUES 
    ('console_theme_light_id', '"light"'),
    ('console_theme_dark_id', '"dark"')
ON CONFLICT (key) DO NOTHING;

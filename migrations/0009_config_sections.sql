-- Migration: Dynamic Config Sections & Fields
-- Created: 2026-01-18
-- Purpose: Enable plugin-extensible config system

-- =============================================
-- 1. CREATE TABLES
-- =============================================

-- Config sections (groupings)
CREATE TABLE IF NOT EXISTS sys_config_sections (
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    code VARCHAR(100) NOT NULL,
    title VARCHAR(255) NOT NULL,
    description TEXT,
    icon VARCHAR(50),
    sort_order INT DEFAULT 0,
    is_owner_only BOOLEAN DEFAULT FALSE,
    plugin_id UUID REFERENCES sys_plugins(id) ON DELETE CASCADE,
    created_at TIMESTAMPTZ DEFAULT NOW()
);

-- Create unique index for code + plugin_id (handles NULL plugin_id correctly)
CREATE UNIQUE INDEX IF NOT EXISTS idx_config_sections_code_plugin 
    ON sys_config_sections(code, plugin_id) WHERE plugin_id IS NOT NULL;
CREATE UNIQUE INDEX IF NOT EXISTS idx_config_sections_code_core 
    ON sys_config_sections(code) WHERE plugin_id IS NULL;

-- Config field definitions
CREATE TABLE IF NOT EXISTS sys_config_fields (
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    section_id UUID NOT NULL REFERENCES sys_config_sections(id) ON DELETE CASCADE,
    key VARCHAR(100) NOT NULL,
    label VARCHAR(255) NOT NULL,
    description TEXT,
    field_type VARCHAR(20) NOT NULL DEFAULT 'text',
    scope VARCHAR(20) DEFAULT 'platform',
    default_value JSONB,
    min_value INT,
    max_value INT,
    options JSONB,
    validation_regex VARCHAR(255),
    sort_order INT DEFAULT 0,
    is_required BOOLEAN DEFAULT FALSE,
    is_secret BOOLEAN DEFAULT FALSE,
    created_at TIMESTAMPTZ DEFAULT NOW(),
    UNIQUE(section_id, key)
);

-- =============================================
-- 2. SEED CORE SECTIONS
-- =============================================

INSERT INTO sys_config_sections (id, code, title, description, icon, sort_order, is_owner_only) VALUES
    ('c1000000-0000-0000-0000-000000000001', 'session_management', 'Session Management', 'Configure token expiration settings', 'clock', 1, TRUE),
    ('c1000000-0000-0000-0000-000000000002', 'rate_limiting', 'API Rate Limiting', 'Control API request throttling', 'shield', 2, TRUE),
    ('c1000000-0000-0000-0000-000000000003', 'ai_automation', 'AI Automation', 'Configure AI translation engine', 'zap', 3, TRUE),
    ('c1000000-0000-0000-0000-000000000004', 'user_interface', 'User Interface', 'Configure UI behavior', 'layout', 4, FALSE),
    ('c1000000-0000-0000-0000-000000000005', 'platform', 'Platform', 'Platform-level configuration', 'server', 5, TRUE)
ON CONFLICT DO NOTHING;

-- =============================================
-- 3. SEED CORE FIELDS
-- =============================================

-- Session Management
INSERT INTO sys_config_fields (section_id, key, label, description, field_type, scope, default_value, min_value, max_value, sort_order, is_required) VALUES
    ('c1000000-0000-0000-0000-000000000001', 'access_token_expire_minutes', 'Access Token (Minutes)', 'How long access tokens remain valid', 'number', 'platform', '30', 5, 1440, 1, TRUE),
    ('c1000000-0000-0000-0000-000000000001', 'refresh_token_expire_minutes', 'Refresh Token (Minutes)', 'How long refresh tokens remain valid', 'number', 'platform', '1440', 60, 43200, 2, TRUE)
ON CONFLICT DO NOTHING;

-- Rate Limiting
INSERT INTO sys_config_fields (section_id, key, label, description, field_type, scope, default_value, min_value, max_value, sort_order, is_required) VALUES
    ('c1000000-0000-0000-0000-000000000002', 'rate_limit_max_requests', 'Max Requests Per Window', 'Maximum API requests allowed per time window', 'number', 'platform', '100', 10, 10000, 1, TRUE),
    ('c1000000-0000-0000-0000-000000000002', 'rate_limit_window_secs', 'Window Duration (Seconds)', 'Time window for rate limiting', 'number', 'platform', '60', 10, 3600, 2, TRUE)
ON CONFLICT DO NOTHING;

-- AI Automation
INSERT INTO sys_config_fields (section_id, key, label, description, field_type, scope, default_value, sort_order, is_required, is_secret, options) VALUES
    ('c1000000-0000-0000-0000-000000000003', 'ai_enabled', 'Enable AI Translation', 'Dynamically translate system labels and content', 'boolean', 'system', 'false', 1, FALSE, FALSE, NULL),
    ('c1000000-0000-0000-0000-000000000003', 'ai_provider', 'Provider', 'AI service provider', 'select', 'system', '"openai"', 2, FALSE, FALSE, '["openai", "deepseek", "qwen"]'),
    ('c1000000-0000-0000-0000-000000000003', 'ai_model', 'Model Identifier', 'Specific model to use', 'text', 'system', '"gpt-4o"', 3, FALSE, FALSE, NULL),
    ('c1000000-0000-0000-0000-000000000003', 'ai_api_key', 'API Key', 'Provider API key', 'secret', 'system', '""', 4, FALSE, TRUE, NULL),
    ('c1000000-0000-0000-0000-000000000003', 'ai_base_url', 'Endpoint URL', 'API endpoint URL', 'text', 'system', '"https://api.openai.com/v1"', 5, FALSE, FALSE, NULL)
ON CONFLICT DO NOTHING;

-- User Interface
INSERT INTO sys_config_fields (section_id, key, label, description, field_type, scope, default_value, sort_order, is_required, options) VALUES
    ('c1000000-0000-0000-0000-000000000004', 'toast_position', 'Toast Notification Placement', 'Where notifications appear on screen', 'select', 'workspace', '"bottom-right"', 1, FALSE, '["top-left", "top-center", "top-right", "bottom-left", "bottom-center", "bottom-right"]')
ON CONFLICT DO NOTHING;

-- Platform
INSERT INTO sys_config_fields (section_id, key, label, description, field_type, scope, default_value, sort_order, is_required, options) VALUES
    ('c1000000-0000-0000-0000-000000000005', 'platform_type', 'Platform Type', 'Single tenant or multi-tenant mode', 'select', 'system', '"multi"', 1, TRUE, '["single", "multi"]')
ON CONFLICT DO NOTHING;

-- =============================================
-- 4. CREATE INDEXES
-- =============================================

CREATE INDEX IF NOT EXISTS idx_config_sections_plugin ON sys_config_sections(plugin_id);
CREATE INDEX IF NOT EXISTS idx_config_fields_section ON sys_config_fields(section_id);
CREATE INDEX IF NOT EXISTS idx_config_fields_key ON sys_config_fields(key);

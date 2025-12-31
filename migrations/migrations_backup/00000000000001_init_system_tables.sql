-- ================================================
-- Migration: 00000000000001_init_system_tables.sql
-- Purpose: Create system-level tables (no FK dependencies)
-- ================================================

-- =====================
-- 1. Audit Logs
-- =====================
CREATE TABLE IF NOT EXISTS audit_logs (
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    timestamp TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    actor TEXT NOT NULL,
    action TEXT NOT NULL,
    target TEXT,
    status TEXT NOT NULL,
    metadata JSONB,
    created_at TIMESTAMPTZ DEFAULT NOW()
);

CREATE INDEX IF NOT EXISTS idx_audit_logs_timestamp ON audit_logs(timestamp);
CREATE INDEX IF NOT EXISTS idx_audit_logs_actor ON audit_logs(actor);

-- =====================
-- 2. System Configs
-- =====================
CREATE TABLE IF NOT EXISTS sys_configs (
    key TEXT PRIMARY KEY,
    value JSONB NOT NULL,
    updated_at TIMESTAMPTZ DEFAULT NOW()
);

DROP TRIGGER IF EXISTS update_sys_configs_updated_at ON sys_configs;
CREATE TRIGGER update_sys_configs_updated_at
BEFORE UPDATE ON sys_configs
FOR EACH ROW EXECUTE FUNCTION update_updated_at_column();

-- =====================
-- 3. API Keys
-- =====================
CREATE TABLE IF NOT EXISTS sys_api_keys (
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    tenant_id VARCHAR(255) NOT NULL,
    key_hash VARCHAR(255) NOT NULL,
    prefix VARCHAR(10) NOT NULL,
    name VARCHAR(255),
    key_type VARCHAR(50) DEFAULT 'server',
    allowed_origins JSONB,
    is_active BOOLEAN DEFAULT TRUE,
    created_at TIMESTAMPTZ DEFAULT NOW(),
    updated_at TIMESTAMPTZ DEFAULT NOW(),
    deleted_at TIMESTAMPTZ
);

DROP TRIGGER IF EXISTS update_sys_api_keys_updated_at ON sys_api_keys;
CREATE TRIGGER update_sys_api_keys_updated_at
BEFORE UPDATE ON sys_api_keys
FOR EACH ROW EXECUTE FUNCTION update_updated_at_column();

-- =====================
-- 4. CORS Origins
-- =====================
CREATE TABLE IF NOT EXISTS sys_cors_origins (
    id SERIAL PRIMARY KEY,
    origin VARCHAR(255) UNIQUE NOT NULL,
    is_active BOOLEAN DEFAULT TRUE,
    description TEXT,
    created_at TIMESTAMPTZ DEFAULT NOW(),
    updated_at TIMESTAMPTZ DEFAULT NOW(),
    deleted_at TIMESTAMPTZ
);

DROP TRIGGER IF EXISTS update_sys_cors_origins_updated_at ON sys_cors_origins;
CREATE TRIGGER update_sys_cors_origins_updated_at
BEFORE UPDATE ON sys_cors_origins
FOR EACH ROW EXECUTE FUNCTION update_updated_at_column();

-- Seed default CORS origins for development
INSERT INTO sys_cors_origins (origin, description) VALUES 
    ('http://localhost:5175', 'Local Platform Development'),
    ('http://localhost:5173', 'Vite Default Port'),
    ('http://localhost:4173', 'Vite Preview Port')
ON CONFLICT (origin) DO NOTHING;

-- =====================
-- 5. i18n Locales
-- =====================
CREATE TABLE IF NOT EXISTS sys_i18n_locales (
    code VARCHAR(10) PRIMARY KEY,
    name VARCHAR(100) NOT NULL,
    is_active BOOLEAN DEFAULT TRUE,
    is_default BOOLEAN DEFAULT FALSE,
    created_at TIMESTAMPTZ DEFAULT NOW(),
    updated_at TIMESTAMPTZ DEFAULT NOW()
);

-- Seed default locales
INSERT INTO sys_i18n_locales (code, name, is_active, is_default) VALUES 
    ('en', 'English', TRUE, TRUE),
    ('th', 'Thai', TRUE, FALSE)
ON CONFLICT (code) DO NOTHING;

-- =====================
-- 6. i18n Translations
-- =====================
CREATE TABLE IF NOT EXISTS sys_i18n_translations (
    id SERIAL PRIMARY KEY,
    locale VARCHAR(10) NOT NULL REFERENCES sys_i18n_locales(code),
    key VARCHAR(255) NOT NULL,
    message TEXT NOT NULL,
    is_auto_generated BOOLEAN DEFAULT FALSE,
    created_at TIMESTAMPTZ DEFAULT NOW(),
    updated_at TIMESTAMPTZ DEFAULT NOW(),
    UNIQUE(locale, key)
);

CREATE INDEX IF NOT EXISTS idx_i18n_translations_locale_key ON sys_i18n_translations(locale, key);

-- ================================================
-- Migration: 0004_i18n_locales.sql
-- Purpose: Internationalization (i18n) locales and translations.
-- ================================================

-- 1. Locales
CREATE TABLE IF NOT EXISTS sys_i18n_locales (
    code VARCHAR(10) PRIMARY KEY,
    name VARCHAR(255) NOT NULL,
    is_active BOOLEAN DEFAULT TRUE,
    is_default BOOLEAN DEFAULT FALSE,
    created_at TIMESTAMPTZ DEFAULT NOW(),
    updated_at TIMESTAMPTZ DEFAULT NOW()
);

CREATE TRIGGER update_sys_i18n_locales_updated_at BEFORE UPDATE ON sys_i18n_locales FOR EACH ROW EXECUTE FUNCTION update_updated_at_column();

-- Seed default locales
INSERT INTO sys_i18n_locales (code, name, is_active, is_default) VALUES 
    ('en', 'English', TRUE, TRUE),
    ('th', 'Thai', TRUE, FALSE)
ON CONFLICT (code) DO NOTHING;

-- 2. Translations
CREATE TABLE IF NOT EXISTS sys_i18n_translations (
    id SERIAL PRIMARY KEY,
    locale VARCHAR(10) NOT NULL REFERENCES sys_i18n_locales(code) ON DELETE CASCADE,
    key VARCHAR(255) NOT NULL,
    message TEXT NOT NULL,
    is_auto_generated BOOLEAN DEFAULT FALSE,
    created_at TIMESTAMPTZ DEFAULT NOW(),
    updated_at TIMESTAMPTZ DEFAULT NOW(),
    UNIQUE(locale, key)
);

CREATE INDEX idx_i18n_translations_locale_key ON sys_i18n_translations(locale, key);
CREATE TRIGGER update_sys_i18n_translations_updated_at BEFORE UPDATE ON sys_i18n_translations FOR EACH ROW EXECUTE FUNCTION update_updated_at_column();

-- 3. Seed Basic Keys (from legacy 20251218000000 and others)
INSERT INTO sys_i18n_translations (locale, key, message) VALUES 
    -- Generic
    ('en', 'common.save', 'Save'),
    ('th', 'common.save', 'บันทึก'),
    ('en', 'common.cancel', 'Cancel'),
    ('th', 'common.cancel', 'ยกเลิก'),
    
    -- Auth/Setup
    ('en', 'setup.welcome', 'Welcome to Kyx Platform'),
    ('th', 'setup.welcome', 'ยินดีต้อนรับสู่ Kyx Platform'),
    
    -- Sidebar
    ('en', 'sidebar.dashboard', 'Dashboard'),
    ('th', 'sidebar.dashboard', 'แดชบอร์ด'),
    ('en', 'sidebar.settings', 'Settings'),
    ('th', 'sidebar.settings', 'ตั้งค่า')
ON CONFLICT (locale, key) DO UPDATE SET message = EXCLUDED.message;

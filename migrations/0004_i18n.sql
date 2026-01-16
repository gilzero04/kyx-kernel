-- ════════════════════════════════════════════════════════════════════════════
-- Migration: 0004_i18n.sql
-- Purpose: Internationalization (i18n) with context separation
-- Consolidated from: 0004, 0026
-- ════════════════════════════════════════════════════════════════════════════

-- ════════════════════════════════════════════════════════════════════════════
-- 1. LOCALES
-- ════════════════════════════════════════════════════════════════════════════

CREATE TABLE IF NOT EXISTS sys_i18n_locales (
    code VARCHAR(10) PRIMARY KEY,
    name VARCHAR(255) NOT NULL,
    is_active BOOLEAN DEFAULT TRUE,
    is_default BOOLEAN DEFAULT FALSE,
    created_at TIMESTAMPTZ DEFAULT NOW(),
    updated_at TIMESTAMPTZ DEFAULT NOW()
);

CREATE TRIGGER update_sys_i18n_locales_updated_at BEFORE UPDATE ON sys_i18n_locales FOR EACH ROW EXECUTE FUNCTION update_updated_at_column();

INSERT INTO sys_i18n_locales (code, name, is_active, is_default) VALUES 
    ('en', 'English', TRUE, TRUE),
    ('th', 'Thai', TRUE, FALSE)
ON CONFLICT (code) DO NOTHING;

-- ════════════════════════════════════════════════════════════════════════════
-- 2. TRANSLATIONS (with context separation)
-- ════════════════════════════════════════════════════════════════════════════

CREATE TABLE IF NOT EXISTS sys_i18n_translations (
    id SERIAL PRIMARY KEY,
    tenant_id UUID REFERENCES auth_tenants(id) ON DELETE CASCADE,
    context VARCHAR(20) CHECK (context IS NULL OR context IN ('console', 'workspace')),
    locale VARCHAR(10) NOT NULL REFERENCES sys_i18n_locales(code) ON DELETE CASCADE,
    key VARCHAR(255) NOT NULL,
    message TEXT NOT NULL,
    is_auto_generated BOOLEAN DEFAULT FALSE,
    created_at TIMESTAMPTZ DEFAULT NOW(),
    updated_at TIMESTAMPTZ DEFAULT NOW()
);

CREATE INDEX idx_i18n_translations_locale_key ON sys_i18n_translations(locale, key);
CREATE INDEX idx_i18n_translations_tenant ON sys_i18n_translations(tenant_id) WHERE tenant_id IS NOT NULL;
CREATE INDEX idx_i18n_translations_context ON sys_i18n_translations(context) WHERE context IS NOT NULL;
CREATE INDEX idx_i18n_translations_tenant_context_locale ON sys_i18n_translations(tenant_id, context, locale);

-- Unique constraint per tenant/context/locale/key
CREATE UNIQUE INDEX idx_i18n_translations_unique 
    ON sys_i18n_translations(locale, key, COALESCE(tenant_id, '00000000-0000-0000-0000-000000000000'::uuid), COALESCE(context, 'global'));

CREATE TRIGGER update_sys_i18n_translations_updated_at BEFORE UPDATE ON sys_i18n_translations FOR EACH ROW EXECUTE FUNCTION update_updated_at_column();

-- ════════════════════════════════════════════════════════════════════════════
-- 3. SEED BASIC TRANSLATIONS
-- ════════════════════════════════════════════════════════════════════════════

INSERT INTO sys_i18n_translations (locale, key, message) VALUES 
    ('en', 'common.save', 'Save'),
    ('th', 'common.save', 'บันทึก'),
    ('en', 'common.cancel', 'Cancel'),
    ('th', 'common.cancel', 'ยกเลิก'),
    ('en', 'setup.welcome', 'Welcome to Kyx Platform'),
    ('th', 'setup.welcome', 'ยินดีต้อนรับสู่ Kyx Platform'),
    ('en', 'sidebar.dashboard', 'Dashboard'),
    ('th', 'sidebar.dashboard', 'แดชบอร์ด'),
    ('en', 'sidebar.settings', 'Settings'),
    ('th', 'sidebar.settings', 'ตั้งค่า')
ON CONFLICT DO NOTHING;

-- ════════════════════════════════════════════════════════════════════════════
-- 4. HELPER VIEW
-- ════════════════════════════════════════════════════════════════════════════

CREATE OR REPLACE VIEW v_i18n_resolved_translations AS
SELECT 
    COALESCE(t.tenant_id, g.tenant_id) as tenant_id,
    COALESCE(t.context, g.context) as context,
    COALESCE(t.locale, g.locale) as locale,
    COALESCE(t.key, g.key) as key,
    COALESCE(t.message, g.message) as message,
    CASE WHEN t.id IS NOT NULL THEN TRUE ELSE FALSE END as is_override
FROM sys_i18n_translations g
LEFT JOIN sys_i18n_translations t ON 
    g.locale = t.locale 
    AND g.key = t.key 
    AND g.tenant_id IS NULL 
    AND t.tenant_id IS NOT NULL
WHERE g.tenant_id IS NULL;

-- ════════════════════════════════════════════════════════════════════════════
-- COMMENTS
-- ════════════════════════════════════════════════════════════════════════════

COMMENT ON TABLE sys_i18n_locales IS 'Available languages/locales';
COMMENT ON TABLE sys_i18n_translations IS 'Translation strings with context separation';
COMMENT ON COLUMN sys_i18n_translations.tenant_id IS 'NULL = global, UUID = tenant-specific override';
COMMENT ON COLUMN sys_i18n_translations.context IS 'NULL = global, console = console-specific, workspace = workspace+app';

-- Create i18n Tables
-- Locales: Supported languages
-- Translations: Key-value translations per locale

-- Locales Table
CREATE TABLE IF NOT EXISTS sys_i18n_locales (
    code VARCHAR(10) PRIMARY KEY,           -- e.g., 'en', 'th', 'ja'
    name VARCHAR(100) NOT NULL,              -- e.g., 'English', 'ภาษาไทย'
    is_active BOOLEAN NOT NULL DEFAULT TRUE,
    is_default BOOLEAN NOT NULL DEFAULT FALSE,
    created_at TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    updated_at TIMESTAMPTZ NOT NULL DEFAULT NOW()
);

-- Translations Table
CREATE TABLE IF NOT EXISTS sys_i18n_translations (
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    locale VARCHAR(10) NOT NULL REFERENCES sys_i18n_locales(code) ON DELETE CASCADE,
    key VARCHAR(255) NOT NULL,               -- e.g., 'core.sidebar.home'
    message TEXT NOT NULL,                   -- Translated text
    is_auto_generated BOOLEAN NOT NULL DEFAULT FALSE,
    created_at TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    updated_at TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    UNIQUE(locale, key)
);

-- Create indices for performance
CREATE INDEX IF NOT EXISTS idx_i18n_translations_locale ON sys_i18n_translations(locale);
CREATE INDEX IF NOT EXISTS idx_i18n_translations_key ON sys_i18n_translations(key);

-- Insert default locales
INSERT INTO sys_i18n_locales (code, name, is_active, is_default) VALUES
('en', 'English', TRUE, TRUE),
('th', 'ภาษาไทย', TRUE, FALSE)
ON CONFLICT (code) DO NOTHING;

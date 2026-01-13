-- Migration: 0026_i18n_context_separation.sql
-- Purpose: Add context separation to i18n (Console separate, Workspace+App shared)
-- Author: AI-generated per architecture design
-- Date: 2026-01-12
-- 
-- i18n Contexts: Console (separate), Workspace (shared with App)

-- ============================================================================
-- 1. Add tenant_id column to sys_i18n_translations
-- ============================================================================

DO $$
BEGIN
    IF NOT EXISTS (
        SELECT 1 FROM information_schema.columns 
        WHERE table_name = 'sys_i18n_translations' AND column_name = 'tenant_id'
    ) THEN
        ALTER TABLE sys_i18n_translations ADD COLUMN tenant_id UUID REFERENCES auth_tenants(id) ON DELETE CASCADE;
    END IF;
END $$;

-- ============================================================================
-- 2. Add context column to sys_i18n_translations
-- ============================================================================

DO $$
BEGIN
    IF NOT EXISTS (
        SELECT 1 FROM information_schema.columns 
        WHERE table_name = 'sys_i18n_translations' AND column_name = 'context'
    ) THEN
        -- context: NULL = global (platform default), 'console' = console-specific, 'workspace' = workspace/app
        ALTER TABLE sys_i18n_translations ADD COLUMN context VARCHAR(20) 
            CHECK (context IS NULL OR context IN ('console', 'workspace'));
    END IF;
END $$;

-- ============================================================================
-- 3. Create indexes for efficient lookup
-- ============================================================================

-- Index for tenant-specific translations lookup
CREATE INDEX IF NOT EXISTS idx_i18n_translations_tenant 
    ON sys_i18n_translations(tenant_id) WHERE tenant_id IS NOT NULL;

-- Index for context-specific translations lookup
CREATE INDEX IF NOT EXISTS idx_i18n_translations_context 
    ON sys_i18n_translations(context) WHERE context IS NOT NULL;

-- Composite index for efficient per-tenant-per-context lookup
CREATE INDEX IF NOT EXISTS idx_i18n_translations_tenant_context_locale 
    ON sys_i18n_translations(tenant_id, context, locale);

-- ============================================================================
-- 4. Update unique constraint to include tenant_id and context
-- ============================================================================

-- Drop old unique constraint if exists
DO $$
BEGIN
    IF EXISTS (
        SELECT 1 FROM pg_constraint 
        WHERE conname = 'sys_i18n_translations_locale_key_key'
    ) THEN
        ALTER TABLE sys_i18n_translations DROP CONSTRAINT sys_i18n_translations_locale_key_key;
    END IF;
END $$;

-- Create new unique constraint that allows same key per tenant/context
CREATE UNIQUE INDEX IF NOT EXISTS idx_i18n_translations_unique 
    ON sys_i18n_translations(locale, key, COALESCE(tenant_id, '00000000-0000-0000-0000-000000000000'::uuid), COALESCE(context, 'global'));

-- ============================================================================
-- 5. Add comments
-- ============================================================================

COMMENT ON COLUMN sys_i18n_translations.tenant_id IS 
    'NULL = global (platform default), UUID = tenant-specific override';
    
COMMENT ON COLUMN sys_i18n_translations.context IS 
    'NULL = global, ''console'' = console-specific, ''workspace'' = workspace+app';

-- ============================================================================
-- 6. Create helper view for translation resolution
-- ============================================================================

CREATE OR REPLACE VIEW v_i18n_resolved_translations AS
SELECT 
    COALESCE(t.tenant_id, g.tenant_id) as tenant_id,
    COALESCE(t.context, g.context) as context,
    COALESCE(t.locale, g.locale) as locale,
    COALESCE(t.key, g.key) as key,
    -- Tenant-specific translation overrides global
    COALESCE(t.message, g.message) as message,
    CASE WHEN t.id IS NOT NULL THEN TRUE ELSE FALSE END as is_override
FROM sys_i18n_translations g
LEFT JOIN sys_i18n_translations t ON 
    g.locale = t.locale 
    AND g.key = t.key 
    AND g.tenant_id IS NULL 
    AND t.tenant_id IS NOT NULL
WHERE g.tenant_id IS NULL;

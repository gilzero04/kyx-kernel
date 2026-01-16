-- ════════════════════════════════════════════════════════════════════════════
-- Migration: Plugin System
-- Purpose: Database schema for WASM-based plugin architecture
-- Compatible with: kyx-engine, pixco-customer-app, external-projects
-- ════════════════════════════════════════════════════════════════════════════

-- ════════════════════════════════════════════════════════════════════════════
-- PLUGIN REGISTRY TABLE
-- ════════════════════════════════════════════════════════════════════════════

CREATE TABLE sys_plugins (
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    tenant_id UUID REFERENCES auth_tenants(id) ON DELETE CASCADE,
    
    -- ─── Plugin Identity ─────────────────────────────────────────────────────
    plugin_id VARCHAR(255) NOT NULL,
    name VARCHAR(255) NOT NULL,
    version VARCHAR(50) NOT NULL,
    description TEXT,
    
    -- ─── Author ──────────────────────────────────────────────────────────────
    author VARCHAR(255),
    author_email VARCHAR(255),
    author_website VARCHAR(512),
    
    -- ─── Display & Branding ──────────────────────────────────────────────────
    icon VARCHAR(255),              -- Emoji or URL
    banner VARCHAR(512),            -- Banner image URL
    category VARCHAR(100),          -- "Entertainment", "Developer Tools"
    tags JSONB NOT NULL DEFAULT '[]',  -- ["live", "streaming"]
    
    -- ─── Links ───────────────────────────────────────────────────────────────
    homepage_url VARCHAR(512),
    documentation_url VARCHAR(512),
    repository_url VARCHAR(512),
    support_url VARCHAR(512),
    
    -- ─── Runtime Configuration ───────────────────────────────────────────────
    runtime VARCHAR(20) NOT NULL DEFAULT 'frontend',
    entry_point VARCHAR(512),       -- "./index.js" or WASM path
    capabilities JSONB NOT NULL DEFAULT '[]',
    permissions JSONB NOT NULL DEFAULT '[]',    -- ["camera", "microphone"]
    data_access JSONB NOT NULL DEFAULT '[]',    -- ["user_profile"]
    network_access JSONB NOT NULL DEFAULT '[]', -- ["api.example.com"]
    config JSONB NOT NULL DEFAULT '{}',
    
    -- ─── WASM Binary Reference ───────────────────────────────────────────────
    wasm_path VARCHAR(512),
    wasm_hash VARCHAR(64),
    wasm_size_bytes BIGINT,
    
    -- ─── Trust & Verification ────────────────────────────────────────────────
    verified BOOLEAN NOT NULL DEFAULT false,
    official BOOLEAN NOT NULL DEFAULT false,
    featured BOOLEAN NOT NULL DEFAULT false,
    is_core_plugin BOOLEAN NOT NULL DEFAULT false,
    
    -- ─── Compatibility ───────────────────────────────────────────────────────
    min_app_version VARCHAR(20),
    max_app_version VARCHAR(20),
    
    -- ─── Status Management ───────────────────────────────────────────────────
    status VARCHAR(20) NOT NULL DEFAULT 'installed',
    is_active BOOLEAN NOT NULL DEFAULT false,
    error_message TEXT,
    
    -- ─── Audit Fields ────────────────────────────────────────────────────────
    installed_at TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    enabled_at TIMESTAMPTZ,
    disabled_at TIMESTAMPTZ,
    created_at TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    updated_at TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    
    -- ─── Constraints ─────────────────────────────────────────────────────────
    CONSTRAINT uq_plugin_tenant UNIQUE(tenant_id, plugin_id),
    CONSTRAINT chk_runtime CHECK (runtime IN ('frontend', 'wasm', 'service', 'hybrid')),
    CONSTRAINT chk_status CHECK (status IN ('installed', 'enabled', 'disabled', 'error', 'updating', 'pending_approval'))
);

-- Indexes
CREATE INDEX idx_plugins_tenant ON sys_plugins(tenant_id);
CREATE INDEX idx_plugins_status ON sys_plugins(status);
CREATE INDEX idx_plugins_active ON sys_plugins(is_active) WHERE is_active = true;
CREATE INDEX idx_plugins_plugin_id ON sys_plugins(plugin_id);
CREATE INDEX idx_plugins_category ON sys_plugins(category);
CREATE INDEX idx_plugins_verified ON sys_plugins(verified) WHERE verified = true;
CREATE INDEX idx_plugins_official ON sys_plugins(official) WHERE official = true;
CREATE INDEX idx_plugins_featured ON sys_plugins(featured) WHERE featured = true;

-- Comment
COMMENT ON TABLE sys_plugins IS 'Plugin registry - compatible with kyx-engine/pixco external plugins';

-- ════════════════════════════════════════════════════════════════════════════
-- PLUGIN EVENTS LOG (Audit Trail)
-- ════════════════════════════════════════════════════════════════════════════

CREATE TABLE sys_plugin_events (
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    plugin_id UUID NOT NULL REFERENCES sys_plugins(id) ON DELETE CASCADE,
    tenant_id UUID NOT NULL REFERENCES auth_tenants(id) ON DELETE CASCADE,
    
    event_type VARCHAR(50) NOT NULL,
    event_data JSONB DEFAULT '{}',
    
    created_at TIMESTAMPTZ NOT NULL DEFAULT NOW()
);

CREATE INDEX idx_plugin_events_plugin ON sys_plugin_events(plugin_id);
CREATE INDEX idx_plugin_events_type ON sys_plugin_events(event_type);
CREATE INDEX idx_plugin_events_created ON sys_plugin_events(created_at);

COMMENT ON TABLE sys_plugin_events IS 'Audit log for plugin lifecycle events';

-- ════════════════════════════════════════════════════════════════════════════
-- PLUGIN PERMISSIONS
-- ════════════════════════════════════════════════════════════════════════════

INSERT INTO sys_permissions (code, slug, name, description, sort_order, is_active, is_system) VALUES
    ('PLG.R', 'plugin:read', 'View Plugins', 'Can view installed plugins', 200, true, true),
    ('PLG.I', 'plugin:install', 'Install Plugins', 'Can install new plugins', 201, true, true),
    ('PLG.E', 'plugin:enable', 'Enable Plugins', 'Can enable/disable plugins', 202, true, true),
    ('PLG.U', 'plugin:uninstall', 'Uninstall Plugins', 'Can remove plugins', 203, true, true),
    ('PLG.C', 'plugin:configure', 'Configure Plugins', 'Can modify plugin settings', 204, true, true),
    ('PLG.A', 'plugin:approve', 'Approve Plugins', 'Can approve plugins with dangerous capabilities', 205, true, true)
ON CONFLICT (slug) DO NOTHING;

-- Grant plugin permissions to superadmin role
INSERT INTO sys_role_permissions (role_id, permission_id)
SELECT r.id, p.id
FROM sys_roles r
CROSS JOIN sys_permissions p
WHERE r.slug = 'superadmin' 
  AND r.tenant_id IS NULL
  AND p.slug IN ('plugin:read', 'plugin:install', 'plugin:enable', 'plugin:uninstall', 'plugin:configure', 'plugin:approve')
ON CONFLICT DO NOTHING;

-- ════════════════════════════════════════════════════════════════════════════
-- UPDATE TRIGGER
-- ════════════════════════════════════════════════════════════════════════════

CREATE OR REPLACE FUNCTION update_plugins_updated_at()
RETURNS TRIGGER AS $$
BEGIN
    NEW.updated_at = NOW();
    RETURN NEW;
END;
$$ LANGUAGE plpgsql;

CREATE TRIGGER trg_plugins_updated_at
    BEFORE UPDATE ON sys_plugins
    FOR EACH ROW
    EXECUTE FUNCTION update_plugins_updated_at();

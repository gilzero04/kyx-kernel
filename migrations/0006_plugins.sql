-- ════════════════════════════════════════════════════════════════════════════
-- Migration: 0006_plugins.sql
-- Purpose: WASM-based plugin architecture
-- Consolidated from: 0017, 0018, 0019
-- ════════════════════════════════════════════════════════════════════════════

-- ════════════════════════════════════════════════════════════════════════════
-- 1. PLUGIN REGISTRY
-- ════════════════════════════════════════════════════════════════════════════

CREATE TABLE sys_plugins (
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    tenant_id UUID REFERENCES auth_tenants(id) ON DELETE CASCADE,
    
    -- Identity
    plugin_id VARCHAR(255) NOT NULL,
    name VARCHAR(255) NOT NULL,
    version VARCHAR(50) NOT NULL,
    description TEXT,
    
    -- Author
    author VARCHAR(255),
    author_email VARCHAR(255),
    author_website VARCHAR(512),
    
    -- Display & Branding
    icon VARCHAR(255),
    banner VARCHAR(512),
    category VARCHAR(100),
    tags JSONB NOT NULL DEFAULT '[]',
    
    -- Links
    homepage_url VARCHAR(512),
    documentation_url VARCHAR(512),
    repository_url VARCHAR(512),
    support_url VARCHAR(512),
    
    -- Runtime Configuration
    runtime VARCHAR(20) NOT NULL DEFAULT 'frontend',
    entry_point VARCHAR(512),
    capabilities JSONB NOT NULL DEFAULT '[]',
    permissions JSONB NOT NULL DEFAULT '[]',
    data_access JSONB NOT NULL DEFAULT '[]',
    network_access JSONB NOT NULL DEFAULT '[]',
    config JSONB NOT NULL DEFAULT '{}',
    ui JSONB DEFAULT NULL,
    
    -- WASM Binary Reference
    wasm_path VARCHAR(512),
    wasm_hash VARCHAR(64),
    wasm_size_bytes BIGINT,
    
    -- Trust & Verification
    verified BOOLEAN NOT NULL DEFAULT false,
    official BOOLEAN NOT NULL DEFAULT false,
    featured BOOLEAN NOT NULL DEFAULT false,
    is_core_plugin BOOLEAN NOT NULL DEFAULT false,
    visibility VARCHAR(20) NOT NULL DEFAULT 'private',
    
    -- Compatibility
    min_app_version VARCHAR(20),
    max_app_version VARCHAR(20),
    
    -- Status Management
    status VARCHAR(20) NOT NULL DEFAULT 'installed',
    is_active BOOLEAN NOT NULL DEFAULT false,
    error_message TEXT,
    
    -- Audit Fields
    installed_at TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    enabled_at TIMESTAMPTZ,
    disabled_at TIMESTAMPTZ,
    created_at TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    updated_at TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    
    -- Constraints
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
CREATE INDEX idx_plugins_visibility ON sys_plugins(visibility);

-- UpdatedAt Trigger
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

-- ════════════════════════════════════════════════════════════════════════════
-- 2. PLUGIN EVENTS LOG
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

-- ════════════════════════════════════════════════════════════════════════════
-- COMMENTS
-- ════════════════════════════════════════════════════════════════════════════

COMMENT ON TABLE sys_plugins IS 'Plugin registry - compatible with kyx-engine/pixco external plugins';
COMMENT ON TABLE sys_plugin_events IS 'Audit log for plugin lifecycle events';
COMMENT ON COLUMN sys_plugins.visibility IS 'Visibility scope: private, shared, global';

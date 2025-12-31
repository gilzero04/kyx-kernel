-- ================================================
-- Migration: 0006_system_refactors.sql
-- Purpose: System-level refactors (e.g., Multitenant Configuration scoping).
-- ================================================

-- 1. Scoped System Configuration (Global vs. Tenant)
-- If tenant_id is NULL, it is a System Default.
ALTER TABLE sys_configs ADD CONSTRAINT fk_sys_configs_tenant FOREIGN KEY (tenant_id) REFERENCES auth_tenants(id) ON DELETE CASCADE;

-- Unique constraint for Tenant-Specific Configs
CREATE UNIQUE INDEX IF NOT EXISTS idx_sys_configs_tenant_key ON sys_configs (key, tenant_id) WHERE tenant_id IS NOT NULL;

-- Unique constraint for Global Configs
CREATE UNIQUE INDEX IF NOT EXISTS idx_sys_configs_global_key ON sys_configs (key) WHERE tenant_id IS NULL;

-- 2. Ensure sys_configs doesn't have a simple PK if we use the (key, tenant_id) combo
-- Note: In 0001 it was created with PK(key). We drop it to allow (key, tenant_id) uniqueness.
ALTER TABLE sys_configs DROP CONSTRAINT IF EXISTS sys_configs_pkey;

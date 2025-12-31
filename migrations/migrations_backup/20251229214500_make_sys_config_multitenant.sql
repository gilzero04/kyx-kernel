-- Migration: 20251229214500_make_sys_config_multitenant.sql
-- Purpose: Add tenant_id to sys_configs to allow storing tenant-specific configurations.
--          If tenant_id is NULL, it is treated as a System Default.

-- 1. Add tenant_id column
ALTER TABLE sys_configs ADD COLUMN IF NOT EXISTS tenant_id UUID REFERENCES auth_tenants(id) ON DELETE CASCADE;

-- 2. Drop existing Primary Key (which was just 'key')
ALTER TABLE sys_configs DROP CONSTRAINT IF EXISTS sys_configs_pkey;

-- 3. Create Unique Indexes instead of a complex PK dealing with NULLs
--    We want (key, tenant_id) to be unique.
--    PostgreSQL < 15 treats NULLs as distinct in unique constraints, allowing duplicates for (key, NULL).
--    But we want ONLY ONE system default per key.
--    So we use partial unique indexes.

-- For Tenant-Specific Configs
CREATE UNIQUE INDEX IF NOT EXISTS idx_sys_configs_tenant_key ON sys_configs (key, tenant_id) WHERE tenant_id IS NOT NULL;

-- For System Default Configs (Global)
CREATE UNIQUE INDEX IF NOT EXISTS idx_sys_configs_global_key ON sys_configs (key) WHERE tenant_id IS NULL;

-- 4. Optionally add an ID column if we ever need to reference rows directly, but (key, tenant_id) is sufficient for lookups.
--    We'll stick to Key lookup for now.

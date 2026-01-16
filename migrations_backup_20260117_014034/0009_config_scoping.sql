-- Migration: config_scoping
-- Description: Add scope field and eliminate NULL tenant_ids in sys_configs

-- 1. Add scope column
ALTER TABLE sys_configs ADD COLUMN IF NOT EXISTS scope TEXT NOT NULL DEFAULT 'platform';

-- 2. Detect Owner ID
DO $$
DECLARE
    owner_id UUID;
BEGIN
    SELECT id INTO owner_id FROM auth_tenants WHERE parent_id IS NULL LIMIT 1;
    
    IF owner_id IS NOT NULL THEN
        -- 3. Update NULL tenant_ids to Owner ID
        UPDATE sys_configs SET tenant_id = owner_id WHERE tenant_id IS NULL;
        
        -- 4. Backfill scopes
        -- Console/Platform scope (Defaults for the whole installation)
        UPDATE sys_configs SET scope = 'platform' 
        WHERE (key LIKE 'console_%' OR key LIKE 'branding_%' OR key LIKE 'theme_%' OR key = 'platform_type')
        AND tenant_id = owner_id;
        
        -- Workspace scope (Tenant-specific or Workspace defaults)
        UPDATE sys_configs SET scope = 'workspace'
        WHERE (key LIKE 'workspace_%')
        AND tenant_id = owner_id;
        
        -- System/Core scope (Internal Infra)
        UPDATE sys_configs SET scope = 'system'
        WHERE key IN ('access_token_expire_minutes', 'refresh_token_expire_minutes')
        AND tenant_id = owner_id;
    END IF;
END $$;

-- 5. Update Unique Constraints
-- Drop existing partial indexes
DROP INDEX IF EXISTS idx_sys_configs_tenant_key;
DROP INDEX IF EXISTS idx_sys_configs_global_key;

-- Add new unified unique constraint
-- We use (key, tenant_id, scope) to allow different values for the same key in different scopes (e.g. platform vs workspace defaults)
ALTER TABLE sys_configs DROP CONSTRAINT IF EXISTS sys_configs_key_tenant_scope_unique;
ALTER TABLE sys_configs ADD CONSTRAINT sys_configs_key_tenant_scope_unique UNIQUE (key, tenant_id, scope);

-- 6. Ensure tenant_id is NOT NULL after migration
ALTER TABLE sys_configs ALTER COLUMN tenant_id SET NOT NULL;

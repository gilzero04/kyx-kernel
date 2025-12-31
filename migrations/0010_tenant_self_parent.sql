-- Migration: tenant_self_parent
-- Description: Update top-level tenants to point to themselves as parent and backfill tenant_id for roles

DO $$
DECLARE
    owner_id UUID;
BEGIN
    -- 1. Find the system owner
    SELECT id INTO owner_id FROM auth_tenants WHERE parent_id IS NULL OR parent_id = id LIMIT 1;
    
    IF owner_id IS NOT NULL THEN
        -- 2. Update existing null parents to be self-referencing
        UPDATE auth_tenants SET parent_id = id WHERE parent_id IS NULL;
        
        -- 3. Backfill NULL tenant_ids for sys_roles and sys_api_keys
        UPDATE sys_roles SET tenant_id = owner_id WHERE tenant_id IS NULL;
        UPDATE sys_api_keys SET tenant_id = owner_id WHERE tenant_id IS NULL;
        
        -- 4. Enforce NOT NULL if data is clean
        -- We do this carefully to avoid breaking fresh installs where owner doesn't exist yet
        EXECUTE 'ALTER TABLE auth_tenants ALTER COLUMN parent_id SET NOT NULL';
        EXECUTE 'ALTER TABLE sys_roles ALTER COLUMN tenant_id SET NOT NULL';
        EXECUTE 'ALTER TABLE sys_api_keys ALTER COLUMN tenant_id SET NOT NULL';
    END IF;
END $$;

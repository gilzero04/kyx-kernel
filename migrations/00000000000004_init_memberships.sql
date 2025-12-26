-- ================================================
-- Migration: 00000000000004_init_memberships.sql
-- Purpose: Create auth_memberships table (user-tenant-role junction)
-- Depends on: auth_users, auth_tenants, sys_roles
-- ================================================

CREATE TABLE IF NOT EXISTS auth_memberships (
    user_id UUID NOT NULL REFERENCES auth_users(id) ON DELETE CASCADE,
    tenant_id UUID NOT NULL REFERENCES auth_tenants(id) ON DELETE CASCADE,
    role VARCHAR(50) NOT NULL,
    role_id UUID REFERENCES sys_roles(id),
    is_active BOOLEAN DEFAULT TRUE,
    created_at TIMESTAMPTZ DEFAULT NOW(),
    updated_at TIMESTAMPTZ DEFAULT NOW(),
    deleted_at TIMESTAMPTZ,
    PRIMARY KEY (user_id, tenant_id)
);

CREATE INDEX IF NOT EXISTS idx_memberships_user_id ON auth_memberships(user_id);
CREATE INDEX IF NOT EXISTS idx_memberships_tenant_id ON auth_memberships(tenant_id);
CREATE INDEX IF NOT EXISTS idx_memberships_role_id ON auth_memberships(role_id);

DROP TRIGGER IF EXISTS update_auth_memberships_updated_at ON auth_memberships;
CREATE TRIGGER update_auth_memberships_updated_at
BEFORE UPDATE ON auth_memberships
FOR EACH ROW EXECUTE FUNCTION update_updated_at_column();

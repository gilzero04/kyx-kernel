# Kyx Kernel: Database Schema Standards

This document defines the **mandatory conventions** for all database tables in the Kyx Kernel.

## 1. Timestamp Fields

| Field | Type | Required For | Auto-Update |
|-------|------|--------------|-------------|
| `created_at` | `TIMESTAMPTZ DEFAULT NOW()` | **All tables** | No (set once on INSERT) |
| `updated_at` | `TIMESTAMPTZ DEFAULT NOW()` | System, Entity tables | **Yes** (via Trigger) |
| `deleted_at` | `TIMESTAMPTZ` (nullable) | System, Entity tables | No (set on soft delete) |

## 2. Table Categories

### System/Reference Tables
> Config, metadata, and reference data. **Must have all 3 timestamp fields + triggers.**

Examples: `sys_roles`, `sys_permissions`, `sys_cors_origins`, `sys_api_keys`

### Entity Tables
> Core business data. **Must have all 3 timestamp fields + triggers.**

Examples: `auth_users`, `auth_tenants`

### Transaction/Log Tables
> Immutable audit records. **Only `created_at`.**

Examples: `audit_logs`

### Join/Mapping Tables
> Many-to-many relationships. **Only `created_at`.**

Examples: `sys_role_permissions`, `auth_memberships`

## 3. Trigger Function (Shared)

All tables with `updated_at` MUST use this trigger:

```sql
CREATE OR REPLACE FUNCTION update_updated_at_column()
RETURNS TRIGGER AS $$
BEGIN
    NEW.updated_at = NOW();
    RETURN NEW;
END;
$$ language 'plpgsql';
```

## 4. Naming Conventions

| Element | Convention | Example |
|---------|------------|---------|
| Tables | `snake_case`, prefixed by module | `auth_users`, `sys_roles` |
| Primary Key | `id UUID` | `id UUID PRIMARY KEY DEFAULT gen_random_uuid()` |
| Foreign Keys | `<entity>_id` | `tenant_id`, `role_id` |
| Triggers | `update_<table>_updated_at` | `update_sys_roles_updated_at` |

## 5. Soft Delete Query Pattern

All queries on soft-deletable tables MUST include:
```sql
WHERE deleted_at IS NULL
```

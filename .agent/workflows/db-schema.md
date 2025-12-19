# Kyx Kernel Database Schema

## ER Diagram

```mermaid
erDiagram
    auth_users {
        uuid id PK
        varchar email UK
        varchar hashed_password
        varchar full_name
        boolean is_active
        timestamptz created_at
        timestamptz updated_at
        timestamptz deleted_at
    }

    auth_tenants {
        uuid id PK
        varchar name
        varchar slug UK
        uuid parent_id FK "Self-reference for hierarchy"
        uuid tenant_type_id FK "Reference to sys_tenant_types"
        boolean is_active
        timestamptz created_at
        timestamptz updated_at
        timestamptz deleted_at
    }

    sys_tenant_types {
        uuid id PK
        varchar slug UK "standard, branch, vendor, supplier"
        varchar name
        text description
        varchar icon
        boolean is_system
        boolean is_active
        timestamptz created_at
        timestamptz updated_at
        timestamptz deleted_at
    }

    auth_memberships {
        uuid user_id PK,FK
        uuid tenant_id PK,FK
        varchar role
        uuid role_id FK
        boolean is_active
        timestamptz created_at
    }

    sys_roles {
        uuid id PK
        varchar name UK
        varchar description
        boolean is_system
        timestamptz created_at
        timestamptz updated_at
        timestamptz deleted_at
    }

    sys_permissions {
        uuid id PK
        varchar slug UK
        varchar description
        varchar category
        timestamptz created_at
        timestamptz updated_at
        timestamptz deleted_at
    }

    sys_role_permissions {
        uuid role_id PK,FK
        uuid permission_id PK,FK
        timestamptz created_at
    }

    sys_api_keys {
        uuid id PK
        varchar tenant_id
        varchar key_hash
        varchar prefix
        varchar name
        varchar key_type
        jsonb allowed_origins
        boolean is_active
        timestamptz created_at
        timestamptz updated_at
        timestamptz deleted_at
    }

    sys_cors_origins {
        serial id PK
        varchar origin UK
        boolean is_active
        text description
        timestamptz created_at
        timestamptz updated_at
        timestamptz deleted_at
    }

    audit_logs {
        uuid id PK
        timestamptz timestamp
        text actor
        text action
        text target
        text status
        jsonb metadata
        timestamptz created_at
    }

    auth_users ||--o{ auth_memberships : "has many"
    auth_tenants ||--o{ auth_memberships : "has many"
    auth_tenants ||--o| auth_tenants : "parent_id (hierarchy)"
    sys_tenant_types ||--o{ auth_tenants : "categorizes"
    sys_roles ||--o{ auth_memberships : "assigned via"
    sys_roles ||--o{ sys_role_permissions : "has many"
    sys_permissions ||--o{ sys_role_permissions : "has many"
```

## Tenant Hierarchy Example

```mermaid
graph TD
    A["🔑 Owner (User: superadmin, tenant_id=NULL)"]
    B["🏢 HQ Tenant (parent_id=NULL)"]
    C["🏪 Branch A (parent_id=HQ)"]
    D["🏪 Branch B (parent_id=HQ)"]
    E["👤 HQ Admin (membership: HQ, superadmin)"]
    F["👤 Branch Admin (membership: Branch A, superadmin)"]
    G["👥 Members (membership: Branch A, viewer)"]

    A -->|manages| B
    B --> C
    B --> D
    B --- E
    C --- F
    C --- G
```

## Table Categories

| Category | Tables | Soft Delete |
|----------|--------|:-----------:|
| **Auth** | auth_users, auth_tenants | ✅ |
| **Join** | auth_memberships, sys_role_permissions | ❌ |
| **System** | sys_roles, sys_permissions, sys_api_keys, sys_cors_origins | ✅ |
| **Transaction** | audit_logs | ❌ |

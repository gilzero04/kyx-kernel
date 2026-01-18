---
description: Entity Data Reference - Complete database schema reference from migrations
---

# Entity Data Reference (EDR) Skill

> **CRITICAL: ต้อง check ก่อนเขียน query ทุกครั้ง! ข้อมูลมาจาก migrations/0001-0010**

## When to Use

- Writing SQL queries in Rust code
- Need to check if table has `deleted_at` column
- Checking table schema before writing INSERT/UPDATE/SELECT
- Debugging "column does not exist" errors

---

## Quick Reference: deleted_at Column

| Table                      | deleted_at | Soft Delete | Notes                            |
| -------------------------- | ---------- | ----------- | -------------------------------- |
| **auth_tenants**           | ✅         | ✅          | Hierarchical multi-tenant        |
| **auth_users**             | ✅         | ✅          | User accounts                    |
| **auth_memberships**       | ✅         | ✅          | User-Tenant-Role mapping         |
| **sys_roles**              | ✅         | ✅          | RBAC roles                       |
| **sys_permissions**        | ✅         | ✅          | RBAC permissions                 |
| **sys_pages**              | ✅         | ✅          | CMS pages                        |
| **sys_cors_origins**       | ✅         | ✅          | CORS whitelist                   |
| **sys_api_keys**           | ✅         | ✅          | API keys                         |
| **sys_tenant_types**       | ✅         | ✅          | Tenant type definitions          |
| **social_posts**           | ✅         | ✅          | Social media posts               |
| **media_assets**           | ✅         | ✅          | Media library assets             |
| ─────────────────────────  | ───        | ───         | ─────────────────────────        |
| **sys_themes**             | ❌         | ❌          | **Hard delete! No soft delete!** |
| **sys_brandings**          | ❌         | ❌          | Hard delete                      |
| **sys_resource_shares**    | ❌         | ❌          | Use `is_active = FALSE`          |
| **sys_plugins**            | ❌         | ❌          | Use `is_active = FALSE`          |
| **sys_i18n_translations**  | ❌         | ❌          | Hard delete                      |
| **sys_i18n_locales**       | ❌         | ❌          | Hard delete                      |
| **user_preferences**       | ❌         | ❌          | Hard delete                      |
| **media_folders**          | ❌         | ❌          | Hard delete                      |
| **sys_workspace_contexts** | ❌         | ❌          | Hard delete                      |
| **audit_logs**             | ❌         | ❌          | Hard delete                      |
| **sys_configs**            | ❌         | ❌          | Hard delete                      |

---

## Query Patterns

### Tables WITH deleted_at

```rust
// ✅ CORRECT: Include deleted_at check
"SELECT ... FROM sys_roles WHERE deleted_at IS NULL AND tenant_id = $1"
"SELECT ... FROM auth_users WHERE deleted_at IS NULL AND id = $1"
"SELECT ... FROM sys_pages WHERE deleted_at IS NULL AND tenant_id = $1"
```

### Tables WITHOUT deleted_at

```rust
// ✅ CORRECT: No deleted_at check
"SELECT ... FROM sys_themes WHERE tenant_id = $1 OR is_shared = TRUE OR is_system = TRUE"
"SELECT ... FROM sys_brandings WHERE tenant_id = $1 AND context = $2"
"SELECT ... FROM sys_resource_shares WHERE is_active = TRUE AND owner_tenant_id = $1"
"SELECT ... FROM sys_plugins WHERE is_active = TRUE"

// ❌ WRONG: These tables don't have deleted_at!
"SELECT ... FROM sys_themes WHERE deleted_at IS NULL"   // WILL ERROR!
"SELECT ... FROM sys_plugins WHERE deleted_at IS NULL"  // WILL ERROR!
```

---

## Complete Table Schemas

### 0001: auth_tenants

```sql
CREATE TABLE auth_tenants (
    id UUID PRIMARY KEY,
    parent_id UUID REFERENCES auth_tenants(id),  -- self-referencing
    tenant_type_id UUID REFERENCES sys_tenant_types(id),
    name VARCHAR(255) NOT NULL,
    slug VARCHAR(255) UNIQUE NOT NULL,
    contact_email VARCHAR(255),
    contact_phone VARCHAR(50),
    website_url TEXT,
    social_links JSONB DEFAULT '{}',
    address TEXT,
    business_type VARCHAR(100),
    custom_domain VARCHAR(255) UNIQUE,
    allow_child_subdomains BOOLEAN DEFAULT FALSE,
    use_parent_subdomain BOOLEAN DEFAULT FALSE,
    domain_verified_at TIMESTAMPTZ,
    verification_token VARCHAR(64) UNIQUE,
    branding_id UUID,
    config JSONB DEFAULT '{}',
    is_active BOOLEAN DEFAULT TRUE,
    created_at TIMESTAMPTZ,
    updated_at TIMESTAMPTZ,
    deleted_at TIMESTAMPTZ  -- ✅ HAS deleted_at
);
```

---

### 0001: auth_users

```sql
CREATE TABLE auth_users (
    id UUID PRIMARY KEY,
    email VARCHAR(255) UNIQUE NOT NULL,
    hashed_password VARCHAR(255) NOT NULL,
    full_name VARCHAR(255),
    avatar_url TEXT,
    cover_url TEXT,
    is_active BOOLEAN DEFAULT TRUE,
    created_at TIMESTAMPTZ,
    updated_at TIMESTAMPTZ,
    deleted_at TIMESTAMPTZ  -- ✅ HAS deleted_at
);
```

---

### 0001: auth_memberships

```sql
CREATE TABLE auth_memberships (
    user_id UUID REFERENCES auth_users(id),
    tenant_id UUID REFERENCES auth_tenants(id),
    role VARCHAR(50) NOT NULL,
    role_id UUID REFERENCES sys_roles(id),
    is_active BOOLEAN DEFAULT TRUE,
    created_at TIMESTAMPTZ,
    updated_at TIMESTAMPTZ,
    deleted_at TIMESTAMPTZ,  -- ✅ HAS deleted_at
    PRIMARY KEY (user_id, tenant_id)
);
```

---

### 0001: sys_roles

```sql
CREATE TABLE sys_roles (
    id UUID PRIMARY KEY,
    tenant_id UUID REFERENCES auth_tenants(id),  -- NULL = global template
    code VARCHAR(50),
    name VARCHAR(255) NOT NULL,
    slug VARCHAR(100) NOT NULL,
    description TEXT,
    landing_path VARCHAR(255),
    sort_order INT DEFAULT 0,
    is_active BOOLEAN DEFAULT TRUE,
    is_shared BOOLEAN DEFAULT FALSE,  -- Added in 0008
    max_members INT,
    created_at TIMESTAMPTZ,
    updated_at TIMESTAMPTZ,
    deleted_at TIMESTAMPTZ,  -- ✅ HAS deleted_at
    UNIQUE(tenant_id, slug)
);
```

---

### 0001: sys_permissions

```sql
CREATE TABLE sys_permissions (
    id UUID PRIMARY KEY,
    code VARCHAR(50) UNIQUE,
    slug VARCHAR(100) UNIQUE NOT NULL,
    name VARCHAR(255) NOT NULL,
    description TEXT,
    sort_order INT DEFAULT 0,
    is_active BOOLEAN DEFAULT TRUE,
    is_system BOOLEAN DEFAULT FALSE,
    created_at TIMESTAMPTZ,
    updated_at TIMESTAMPTZ,
    deleted_at TIMESTAMPTZ  -- ✅ HAS deleted_at
);
```

---

### 0003: sys_themes ⚠️

```sql
CREATE TABLE sys_themes (
    id UUID PRIMARY KEY,
    tenant_id UUID REFERENCES auth_tenants(id),
    name VARCHAR(255) NOT NULL,
    slug VARCHAR(100) UNIQUE,
    type VARCHAR(20) DEFAULT 'dark',  -- 'light' | 'dark'
    description TEXT,
    config JSONB NOT NULL DEFAULT '{}',
    version VARCHAR(20) DEFAULT '1.0.0',
    author VARCHAR(255),
    preview_url TEXT,
    logo_url TEXT,
    is_active BOOLEAN DEFAULT TRUE,
    is_system BOOLEAN DEFAULT FALSE,
    is_shared BOOLEAN DEFAULT FALSE,  -- Added in 0008
    created_at TIMESTAMPTZ,
    updated_at TIMESTAMPTZ
    -- ⚠️ NO deleted_at! Hard delete only!
    -- ⚠️ visibility column was DROPPED in 0009!
);
```

---

### 0005: sys_pages

```sql
CREATE TABLE sys_pages (
    id UUID PRIMARY KEY,
    tenant_id UUID NOT NULL REFERENCES auth_tenants(id),
    slug VARCHAR(255) NOT NULL,
    title VARCHAR(255) NOT NULL,
    content JSONB DEFAULT '[]',
    is_published BOOLEAN DEFAULT FALSE,
    is_shared BOOLEAN DEFAULT FALSE,  -- Added in 0008
    created_at TIMESTAMPTZ,
    updated_at TIMESTAMPTZ,
    deleted_at TIMESTAMPTZ,  -- ✅ HAS deleted_at
    UNIQUE(tenant_id, slug)
);
```

---

### 0005: media_assets

```sql
CREATE TABLE media_assets (
    id UUID PRIMARY KEY,
    tenant_id UUID NOT NULL REFERENCES auth_tenants(id),
    folder_id UUID REFERENCES media_folders(id),
    filename VARCHAR(255) NOT NULL,
    original_name VARCHAR(255) NOT NULL,
    mime_type VARCHAR(100) NOT NULL,
    file_size BIGINT NOT NULL,
    url TEXT NOT NULL,
    visibility VARCHAR(20) DEFAULT 'private',
    is_shared BOOLEAN DEFAULT FALSE,  -- Added in 0008
    metadata JSONB DEFAULT '{}',
    created_at TIMESTAMPTZ,
    updated_at TIMESTAMPTZ,
    deleted_at TIMESTAMPTZ  -- ✅ HAS deleted_at
);
```

---

### 0006: sys_plugins

```sql
CREATE TABLE sys_plugins (
    id UUID PRIMARY KEY,
    tenant_id UUID REFERENCES auth_tenants(id),
    plugin_id VARCHAR(255) NOT NULL,
    name VARCHAR(255) NOT NULL,
    version VARCHAR(50) NOT NULL,
    description TEXT,
    author VARCHAR(255),
    -- ... (more fields)
    status VARCHAR(20) DEFAULT 'installed',
    is_active BOOLEAN DEFAULT FALSE,  -- Use for "soft disable"
    error_message TEXT,
    installed_at TIMESTAMPTZ,
    enabled_at TIMESTAMPTZ,
    disabled_at TIMESTAMPTZ,
    created_at TIMESTAMPTZ,
    updated_at TIMESTAMPTZ
    -- ⚠️ NO deleted_at! Use is_active = FALSE
);
```

---

### 0007: sys_brandings

```sql
CREATE TABLE sys_brandings (
    id UUID PRIMARY KEY,
    tenant_id UUID REFERENCES auth_tenants(id),
    context VARCHAR(20) DEFAULT 'workspace',  -- 'console' | 'workspace'
    name VARCHAR(255) NOT NULL,
    description TEXT,
    logo_light_url TEXT,
    logo_dark_url TEXT,
    favicon_url TEXT,
    icon_app_url TEXT,
    splash_image_url TEXT,
    app_name VARCHAR(255),
    splash_text VARCHAR(255),
    splash_subtext VARCHAR(255),
    tagline VARCHAR(255),
    theme_light_id UUID REFERENCES sys_themes(id),
    theme_dark_id UUID REFERENCES sys_themes(id),
    theme_workspace_light_id UUID REFERENCES sys_themes(id),
    theme_workspace_dark_id UUID REFERENCES sys_themes(id),
    theme_app_light_id UUID REFERENCES sys_themes(id),
    theme_app_dark_id UUID REFERENCES sys_themes(id),
    metadata JSONB DEFAULT '{}',
    created_at TIMESTAMPTZ,
    updated_at TIMESTAMPTZ
    -- ⚠️ NO deleted_at! Hard delete only
);
```

---

### 0008: sys_resource_shares

```sql
CREATE TABLE sys_resource_shares (
    id UUID PRIMARY KEY,
    resource_type VARCHAR(30) NOT NULL,  -- 'role' | 'theme' | 'page' | 'media' | 'permission'
    resource_id UUID NOT NULL,
    owner_tenant_id UUID NOT NULL REFERENCES auth_tenants(id),
    shared_to_tenant_id UUID NOT NULL REFERENCES auth_tenants(id),
    can_reshare BOOLEAN DEFAULT FALSE,
    is_active BOOLEAN DEFAULT TRUE,  -- Use for "soft revoke"
    created_by UUID REFERENCES auth_users(id),
    created_at TIMESTAMPTZ,
    updated_at TIMESTAMPTZ
    -- ⚠️ NO deleted_at! Use is_active = FALSE
);
```

---

## Validation Checklist

Before writing any SQL query:

- [ ] Check if table is in "WITH deleted_at" list
- [ ] For tables WITHOUT deleted_at, use `is_active = FALSE` or hard delete
- [ ] Never assume `deleted_at` exists - **verify first!**
- [ ] For sys_themes: Never use `deleted_at`, `visibility` (both don't exist)

---

## Quick Lookup by File

| Repository File | Tables Used                     | Has deleted_at |
| --------------- | ------------------------------- | -------------- |
| `theme/mod.rs`  | sys_themes                      | ❌ NO          |
| `role_query.rs` | sys_roles                       | ✅ YES         |
| `share/mod.rs`  | sys_resource_shares, sys_themes | ❌ NO          |
| `cms/mod.rs`    | sys_pages                       | ✅ YES         |
| `user/mod.rs`   | auth_users, auth_memberships    | ✅ YES         |
| `tenant/mod.rs` | auth_tenants                    | ✅ YES         |

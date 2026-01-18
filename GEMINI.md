# Kyx Kernel — Project-Specific Rules

> **Global Rules**: See `~/.gemini/GEMINI.md` for cross-project Prime Directives, Auto-Run permissions, and Branding/Theme rules.

---

## 🏗️ Project: kyx-kernel

### Plugin Architecture

- **Isolation**: Plugins must use `plugin_*` database prefixes.
- **Independence**: All plugins (except `kyx-face`) must function independently.
- **Integrity**: Core must function 100% without any optional plugins.

### Theme System Integration

- Use `slug` field from `manifest.json` as human-readable identifier
- Use `id` (UUID) for database matching
- **Sharing**: Use `is_shared` column (NOT visibility!)

---

## 🔗 Resource Sharing (RULE 19)

### Shareable Resources in Kernel

| Resource     | Table                   | Query File      |
| ------------ | ----------------------- | --------------- |
| Roles        | `sys_roles`             | `role_query.rs` |
| Themes       | `sys_themes`            | `theme/mod.rs`  |
| Pages        | `sys_pages`             | `cms/mod.rs`    |
| Media        | `media_assets`          | `media/mod.rs`  |
| Translations | `sys_i18n_translations` | `i18n/mod.rs`   |

### Sharing Query Pattern

```rust
// For tables WITH deleted_at (auth_tenants, auth_users, sys_roles, etc.)
WHERE deleted_at IS NULL AND (
    tenant_id = $1
    OR can_access_shared_resource('resource_type', id, $1)
)

// For tables WITHOUT deleted_at (sys_themes, media_assets, etc.)
WHERE tenant_id = $1 OR is_shared = TRUE OR is_system = TRUE
```

---

## 📋 Entity Data Reference (EDR)

> **CRITICAL: ต้อง check ก่อนเขียน query ทุกครั้ง! ข้อมูลมาจาก migrations/0001-0010**

### Quick Reference: deleted_at Column

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
    deleted_at TIMESTAMPTZ  -- ✅ HAS deleted_at (per 0005)
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
    ...
    status VARCHAR(20) DEFAULT 'installed',
    is_active BOOLEAN DEFAULT FALSE,  -- Use this for "soft disable"
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
    is_active BOOLEAN DEFAULT TRUE,  -- Use this for "soft revoke"
    created_by UUID REFERENCES auth_users(id),
    created_at TIMESTAMPTZ,
    updated_at TIMESTAMPTZ
    -- ⚠️ NO deleted_at! Use is_active = FALSE
);
```

---

### Query Examples

```rust
// ✅ CORRECT: sys_themes (no deleted_at)
"SELECT ... FROM sys_themes WHERE tenant_id = $1 OR is_shared = TRUE OR is_system = TRUE"

// ✅ CORRECT: sys_roles (has deleted_at)
"SELECT ... FROM sys_roles WHERE deleted_at IS NULL AND tenant_id = $1"

// ✅ CORRECT: sys_resource_shares (no deleted_at, use is_active)
"SELECT ... FROM sys_resource_shares WHERE is_active = TRUE AND owner_tenant_id = $1"

// ❌ WRONG: sys_themes with deleted_at
"SELECT ... FROM sys_themes WHERE deleted_at IS NULL"

// ❌ WRONG: sys_plugins with deleted_at
"SELECT ... FROM sys_plugins WHERE deleted_at IS NULL"
```

### API Endpoints

| Method   | Endpoint                   | Handler File   |
| -------- | -------------------------- | -------------- |
| `GET`    | `/admin/shares`            | `share/mod.rs` |
| `GET`    | `/admin/shares/received`   | `share/mod.rs` |
| `POST`   | `/admin/shares`            | `share/mod.rs` |
| `DELETE` | `/admin/shares/{id}`       | `share/mod.rs` |
| `GET`    | `/admin/shares/{id}/usage` | `share/mod.rs` |

---

## 🎨 Theme System Architecture

> **สถาปัตยกรรมการจัดเก็บและให้บริการ Themes**

### การจัดเก็บ Theme

```
┌─────────────────────────────────────────────────────────────┐
│               Theme Storage Architecture                     │
├─────────────────────────────────────────────────────────────┤
│                                                              │
│  1️⃣ Compile Time (Development)                              │
│  └── assets/themes/presets/kyx-dark/                        │
│       ├── manifest.json    ← Theme metadata + config         │
│       ├── theme.css        ← Main entry (imports others)    │
│       ├── button.css       ← Component styles               │
│       └── images/          ← Preview, logo, etc.            │
│                                                              │
│  2️⃣ Runtime Seed → Database (PostgreSQL)                    │
│  └── sys_themes table:                                       │
│       ├── id, code, name   ← Identification                 │
│       ├── config.theme_css ← ⭐ BUNDLED CSS STRING           │
│       ├── preview_url      ← Path to static image           │
│       └── logo_url         ← Path to static image           │
│                                                              │
│  3️⃣ Static File Serving (Images Only)                       │
│  └── /themes/presets/{theme}/ → ./assets/themes/presets/    │
│       └── Serves: preview.png, logo.png                     │
│                                                              │
└─────────────────────────────────────────────────────────────┘
```

### Theme ID Format

| Field           | Format            | Example                                |
| --------------- | ----------------- | -------------------------------------- |
| `id` (manifest) | `UUID`            | `6f29f480-e8bb-4fb8-b03f-8c3859544891` |
| `id` (database) | `UUID`            | `6f29f480-e8bb-4fb8-b03f-8c3859544891` |
| `slug`          | `{vendor}-{name}` | `kyx-dark`, `pixco-ocean`              |

> **Note:** `id` ใน manifest และ database ใช้ plain UUID เหมือนกัน ไม่ต้องแปลง

### CSS Bundling Process

```mermaid
graph LR
    A[theme.css] --> B[inline_css_recursive]
    B --> C[@import './button.css']
    B --> D[@import './card.css']
    C --> E[Inline to single string]
    D --> E
    E --> F[config.theme_css in DB]
```

1. **`seed_default_themes()`** reads `assets/themes/presets/*/manifest.json`
2. **`inline_css_recursive()`** inlines all `@import` statements
3. **Bundled CSS** stored in `sys_themes.config.theme_css`
4. **Frontend** fetches theme from API and applies CSS dynamically

### Docker Build (kyx-kernel)

**ไม่ต้อง COPY assets/ ไป Docker** เพราะ:

- Themes ถูก bundle และเก็บใน DB ตอน seed
- Images ให้บริการผ่าน CDN/Frontend

**แต่ถ้าต้องการ Static File Serving:**

```dockerfile
# Only if serving theme images from backend
COPY assets ./assets
```

### Theme File Sync (Development)

**Source of Truth:** `kyx-platform/static/themes/presets/`

**Sync Script:** `kyx-kernel/scripts/sync-themes.sh`

```bash
# From kyx-kernel directory
./scripts/sync-themes.sh
```

**Workflow:**

1. แก้ไข themes ที่ `kyx-platform`
2. รัน `sync-themes.sh` จาก `kyx-kernel`
3. Commit ทั้งสอง repos

### Related Files

| `src/modules/system/application/services/theme/mod.rs` | Theme service + seeding |
| `src/modules/system/interface/http/handlers/theme/mod.rs` | Theme API handlers |
| `migrations/0028_seed_default_themes.sql` | DB seed migration |

---

## 🎨 Branding Setup Rules

> **Critical Ordering for Setup**

### Creation Order (FK Constraint)

```
1️⃣ Create tenant (WITHOUT branding_id)
   └── INSERT INTO auth_tenants (id, parent_id, name, slug, tenant_type_id)

2️⃣ Create branding records (tenant now exists, FK passes)
   └── INSERT INTO sys_brandings (id, tenant_id, context, ...)
   └── 2 records: context='console' AND context='workspace'

3️⃣ Update tenant with branding_id
   └── UPDATE auth_tenants SET branding_id = ? WHERE id = ?
```

### Branding Record Requirements

| Tenant Type             | Required Records                            |
| ----------------------- | ------------------------------------------- |
| Owner/Business (Parent) | `context='console'` + `context='workspace'` |
| Branch/Vendor (Child)   | `context='workspace'` only                  |

**Record Rules:**

- `name`: Use org_name directly (NO " Branding" suffix!)
- `tenant_id`: MUST be set on all records

### Context-Specific Theme Columns

| Context       | Columns Used                       | Columns NOT Used      |
| ------------- | ---------------------------------- | --------------------- |
| **Console**   | `theme_light_id`, `theme_dark_id`  | workspace/app columns |
| **Workspace** | `theme_workspace_*`, `theme_app_*` | theme_light/dark_id   |

### Fetching & Saving Rules

```sql
-- FETCHING: Always filter by context!
SELECT theme_light_id, theme_dark_id FROM sys_brandings
WHERE tenant_id = $1 AND context = 'console'

-- SAVING: Update only context-specific columns!
UPDATE sys_brandings SET theme_light_id = $1
WHERE tenant_id = $2 AND context = 'console'
```

### Theme Mode Assignment

> **Any theme can be assigned to Light or Dark mode!**

The theme name does NOT determine which mode it's used for:

- `theme_light_id` = Theme for Light Mode (could be `kyx-dark` if desired!)
- `theme_dark_id` = Theme for Dark Mode (could be `kyx-light` if desired!)

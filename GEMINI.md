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
// Include shared resources in list query
WHERE deleted_at IS NULL AND (
    tenant_id = $1
    OR can_access_shared_resource('resource_type', id, $1)
)
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

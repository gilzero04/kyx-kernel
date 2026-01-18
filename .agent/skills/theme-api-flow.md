---
description: Theme API Flow - Complete backend process for theme loading and management
---

# Theme API Flow Skill

> สรุปกระบวนการทำงานของ Theme API ใน kyx-kernel

## When to Use

- ต้องการเข้าใจ flow การดึง/จัดการ themes
- Debug theme loading issues
- เพิ่ม feature ใหม่เกี่ยวกับ themes

---

## High-Level Architecture

```
┌─────────────────────────────────────────────────────────────────────────────┐
│                           Theme API Flow                                     │
├─────────────────────────────────────────────────────────────────────────────┤
│                                                                              │
│  Frontend (kyx-platform)                                                     │
│  └── ThemeManager.ts                                                         │
│       ├── fetchPublicThemes()  ─────────────────────────┐                   │
│       └── activateTheme()      ─────────────────────────┼─┐                 │
│                                                          │ │                 │
│  ════════════════════════════════════════════════════════╪═╪═════════════   │
│                                                          │ │                 │
│  Backend (kyx-kernel)                                   ▼ ▼                 │
│  └── HTTP Layer (routers/theme/mod.rs)                                      │
│       ├── GET  /api/v1/public/themes  ──► list_themes (PUBLIC)              │
│       ├── POST /admin/themes/import   ──► import_theme                      │
│       ├── PATCH /admin/themes/{id}/activate ──► set_active_theme            │
│       ├── PATCH /admin/themes/{id}/sharing  ──► set_sharing                 │
│       ├── POST /admin/themes/{id}/update    ──► update_theme                │
│       └── DELETE /admin/themes/{id}         ──► delete_theme                │
│                                                                              │
│  └── Handler Layer (handlers/theme/mod.rs)                                   │
│       └── list_themes()                                                      │
│            ├── Extract Claims (optional) from JWT                           │
│            ├── Get tenant_id from claims                                    │
│            └── Call service.list_available_themes(tenant_id)                │
│                                                                              │
│  └── Service Layer (services/theme/mod.rs)                                   │
│       └── list_available_themes(tenant_id)                                   │
│            └── Call repo.find_available(tenant_id)                          │
│                                                                              │
│  └── Repository Layer (repositories/theme/mod.rs)                            │
│       └── find_available(tenant_id)                                          │
│            ├── IF tenant_id is Some:                                        │
│            │    SELECT ... WHERE tenant_id = $1                             │
│            │                   OR is_shared = TRUE                          │
│            │                   OR is_system = TRUE                          │
│            └── ELSE (None = Admin):                                         │
│                 SELECT * FROM sys_themes (ALL)                              │
│                                                                              │
│  └── Database (PostgreSQL)                                                   │
│       └── sys_themes table                                                   │
│            ├── id, slug, name, type                                         │
│            ├── config (JSONB with theme_css)                                │
│            ├── is_shared, is_system, is_active                              │
│            └── ⚠️ NO deleted_at column!                                     │
│                                                                              │
└─────────────────────────────────────────────────────────────────────────────┘
```

---

## Endpoint Reference

### Public Endpoints (No Auth)

| Method | Path                    | Handler       | Description           |
| ------ | ----------------------- | ------------- | --------------------- |
| GET    | `/api/v1/public/themes` | `list_themes` | List available themes |

### Protected Endpoints (Require Auth + Permission)

| Method | Path                          | Handler            | Permission       |
| ------ | ----------------------------- | ------------------ | ---------------- |
| POST   | `/admin/themes/import`        | `import_theme`     | `theme:import`   |
| PATCH  | `/admin/themes/{id}/activate` | `set_active_theme` | `theme:activate` |
| PATCH  | `/admin/themes/{id}/sharing`  | `set_sharing`      | `theme:update`   |
| POST   | `/admin/themes/{id}/update`   | `update_theme`     | `theme:update`   |
| DELETE | `/admin/themes/{id}`          | `delete_theme`     | `theme:delete`   |

---

## Theme Visibility Logic

```rust
// repositories/theme/mod.rs:60-86

fn find_available(tenant_id: Option<Uuid>) {
    if let Some(tid) = tenant_id {
        // Tenant sees:
        // 1. Own themes (tenant_id = $1)
        // 2. Shared themes (is_shared = TRUE)
        // 3. System themes (is_system = TRUE)
        SELECT ... WHERE tenant_id = $1
                      OR is_shared = TRUE
                      OR is_system = TRUE
    } else {
        // Admin sees ALL themes
        SELECT * FROM sys_themes
    }
}
```

### Visibility Matrix

> **Note:** `tenant_id` ไม่ควรเป็น NULL - Theme ทุกอันต้องมี owner (tenant_id) อย่างน้อยก็เป็นของ Platform Owner

| Theme Type      | tenant_id   | is_shared | is_system | Visible To                    |
| --------------- | ----------- | --------- | --------- | ----------------------------- |
| System (Kyx)    | Owner UUID  | TRUE      | TRUE      | Everyone (seeded by platform) |
| Shared (Custom) | Tenant UUID | TRUE      | FALSE     | Owner + Descendants           |
| Private         | Tenant UUID | FALSE     | FALSE     | Owner only                    |

**Rules:**

- `is_system = TRUE` → Official Kyx themes, seeded during setup, owned by Platform Owner
- `is_shared = TRUE` → Broadcast to all descendants
- Both FALSE → Private to owner only

---

## Theme Import Flow

```
1. User uploads ZIP file via POST /admin/themes/import
   │
2. Handler: import_theme() (handlers/theme/mod.rs:246-340)
   ├── Extract multipart payload
   ├── Read ZIP file bytes
   └── Call extract_theme_config(bytes)
       │
3. extract_theme_config() (handlers/theme/mod.rs:85-135)
   ├── Parse ZIP archive
   ├── Read manifest.json
   ├── Bundle CSS recursively (resolve @imports)
   ├── Convert images to base64 data URLs
   └── Return merged config with theme_css
       │
4. Service: create_theme(dto)
   │
5. Repository: INSERT INTO sys_themes
   │
6. Return Theme object with bundled CSS
```

---

## CSS Bundling Process

```rust
// handlers/theme/mod.rs:38-82

fn bundle_css_from_zip(archive, file_name, visited) {
    // 1. Read file content from ZIP
    // 2. For each line:
    //    - If @import './xyz.css':
    //      - Recursively inline that file
    //    - Else: keep line as-is
    // 3. Return single bundled CSS string
}
```

**Example:**

```css
/* theme.css */
@import './variables.css';
@import './button.css';

/* After bundling: */
:root { --color-primary: #fff; }
.btn { ... }
```

---

## Theme Activation Flow

```
1. PATCH /admin/themes/{id}/activate
   │
2. Handler: set_active_theme() (handlers/theme/mod.rs:343-430)
   ├── Verify theme exists
   ├── Verify tenant has access to theme
   ├── Get branding_id from tenant
   ├── Update sys_brandings (theme_light_id or theme_dark_id based on mode)
   └── Return success
       │
3. Frontend: Update localStorage + refreshBranding()
```

---

## Key Files Reference

| Layer      | File                        | Purpose                                   |
| ---------- | --------------------------- | ----------------------------------------- |
| Router     | `routers/theme/mod.rs`      | Route definitions + permission middleware |
| Handler    | `handlers/theme/mod.rs`     | HTTP request handling, ZIP extraction     |
| Service    | `services/theme/mod.rs`     | Business logic                            |
| Repository | `repositories/theme/mod.rs` | Database queries                          |
| Domain     | `domain/theme.rs`           | Theme entity + trait definitions          |
| DTO        | `dto/theme.rs`              | Data transfer objects                     |

---

## Critical Notes

### ⚠️ sys_themes Has NO deleted_at!

```rust
// ❌ WRONG - Will cause 500 error!
"SELECT ... FROM sys_themes WHERE deleted_at IS NULL"

// ✅ CORRECT
"SELECT ... FROM sys_themes WHERE tenant_id = $1 OR is_shared = TRUE"
```

### ⚠️ visibility Column Was DROPPED!

```sql
-- Migration 0009: visibility column removed
-- Use is_shared instead
ALTER TABLE sys_themes DROP COLUMN IF EXISTS visibility;
```

---

## Error Handling

| Error                   | Cause                      | Fix                         |
| ----------------------- | -------------------------- | --------------------------- |
| 500 on `/public/themes` | Query uses `deleted_at`    | Remove from query           |
| 404 Theme Not Found     | Incorrect ID or visibility | Check is_shared/is_system   |
| 403 Forbidden           | No permission              | Check role permissions      |
| 400 Invalid ZIP         | Bad theme package          | Verify manifest.json exists |

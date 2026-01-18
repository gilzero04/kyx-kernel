# KYX Kernel API Reference

> **Version**: 0.1.0  
> **Base URL**: `/api/v1`  
> **Last Updated**: 2026-01-18

---

## 🔐 Standard Response Format

All endpoints return `ApiResponse`:

```json
{
  "success": true,
  "status": 200,
  "message": "Description",
  "data": { ... },
  "meta": { "timestamp": "2026-01-18T11:00:00Z" }
}
```

---

## 📌 Public Endpoints (No Auth Required)

### Branding

| Method | Endpoint           | Description                            |
| ------ | ------------------ | -------------------------------------- |
| GET    | `/public/branding` | Get tenant branding (unified endpoint) |

**Query Parameters:**
| Param | Type | Description |
|-------|------|-------------|
| `tenant_id` | UUID | Lookup by tenant ID |
| `slug` | String | Lookup by tenant slug |
| `domain` | String | Lookup by custom domain |
| `context` | String | `console` or `workspace` (default: console) |

**Response:**

```json
{
  "tenant": {"id", "name", "slug", "type", "is_active"},
  "branding": {
    "app_name", "logo", "logo_dark", "favicon", "icon_app",
    "splash": {"text", "subtext"},
    "themes": {
      "console": {"light": "kyx-light", "dark": "kyx-dark"},
      "workspace": {"light", "dark"},
      "app": {"light", "dark"}
    }
  },
  "context": "console"
}
```

---

### System

| Method | Endpoint                | Description                        |
| ------ | ----------------------- | ---------------------------------- |
| GET    | `/public/system/info`   | Build info (version, commit, date) |
| GET    | `/public/system/status` | System health status               |

---

### i18n

| Method | Endpoint                | Description            |
| ------ | ----------------------- | ---------------------- |
| GET    | `/public/i18n/locales`  | List available locales |
| GET    | `/public/i18n/{locale}` | Get translations       |

---

### Themes

| Method | Endpoint         | Description     |
| ------ | ---------------- | --------------- |
| GET    | `/public/themes` | List all themes |

---

## 🔒 Admin Endpoints (Auth Required)

### RBAC

| Method | Endpoint                  | Description       |
| ------ | ------------------------- | ----------------- |
| GET    | `/admin/roles`            | List roles        |
| POST   | `/admin/roles`            | Create role       |
| PATCH  | `/admin/roles/{id}`       | Update role       |
| DELETE | `/admin/roles/{id}`       | Delete role       |
| GET    | `/admin/permissions`      | List permissions  |
| POST   | `/admin/permissions`      | Create permission |
| PATCH  | `/admin/permissions/{id}` | Update permission |
| DELETE | `/admin/permissions/{id}` | Delete permission |

---

### Tenants

| Method | Endpoint            | Description         |
| ------ | ------------------- | ------------------- |
| GET    | `/admin/tenants`    | List tenants        |
| GET    | `/admin/tenants/me` | Current tenant info |

---

### CORS

| Method | Endpoint           | Description       |
| ------ | ------------------ | ----------------- |
| GET    | `/admin/cors`      | List CORS origins |
| POST   | `/admin/cors`      | Add origin        |
| PATCH  | `/admin/cors/{id}` | Update origin     |
| DELETE | `/admin/cors/{id}` | Delete origin     |

---

### Config

| Method | Endpoint        | Description    |
| ------ | --------------- | -------------- |
| GET    | `/admin/config` | Get all config |
| PATCH  | `/admin/config` | Update config  |

---

### Context

| Method | Endpoint         | Description                      |
| ------ | ---------------- | -------------------------------- |
| GET    | `/admin/context` | Get owner_id for hierarchy check |

---

### Themes (Admin)

| Method | Endpoint                      | Description                  |
| ------ | ----------------------------- | ---------------------------- |
| POST   | `/admin/themes/import`        | Import theme from ZIP        |
| PATCH  | `/admin/themes/{id}/activate` | Activate theme for mode      |
| PATCH  | `/admin/themes/{id}/sharing`  | Update theme sharing status  |
| POST   | `/admin/themes/{id}/update`   | Update system theme from ZIP |
| DELETE | `/admin/themes/{id}`          | Delete theme                 |

---

## 🔑 Authentication

| Method | Endpoint        | Description         |
| ------ | --------------- | ------------------- |
| POST   | `/auth/login`   | Login (returns JWT) |
| POST   | `/auth/logout`  | Logout              |
| POST   | `/auth/refresh` | Refresh token       |

**Login Request:**

```json
{ "email": "admin@example.com", "password": "..." }
```

**Login Response:**

```json
{
  "data": {
    "user": {...},
    "tokens": {"access_token", "refresh_token"},
    "tenant": {...}
  }
}
```

---

## 📝 Changelog

| Date       | Changes                                         |
| ---------- | ----------------------------------------------- |
| 2026-01-18 | Added Theme admin endpoints (ApiResponse)       |
| 2026-01-18 | Added unified `/public/branding` endpoint       |
| 2026-01-18 | Added ApiResponse wrapper to all handlers       |
| 2026-01-18 | Removed color fields from branding (use themes) |

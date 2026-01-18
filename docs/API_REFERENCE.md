# Kyx Kernel API Reference

> **Base URL:** `http://localhost:8080` (dev) | `https://api.kyx.io` (prod)  
> **Authentication:** Bearer JWT Token  
> **Content-Type:** `application/json`

---

## Quick Navigation

- [Public Endpoints](#public-endpoints-no-auth)
- [Auth Endpoints](#auth-endpoints)
- [Me Endpoints](#me-endpoints-user-context)
- [Admin Endpoints](#admin-endpoints)
- [Media Endpoints](#media-endpoints)
- [Signal Endpoints](#signal-endpoints-kyx-signal-integration)

---

## Public Endpoints (No Auth)

### System Info

| Method | Endpoint                       | Description                       |
| ------ | ------------------------------ | --------------------------------- |
| GET    | `/api/v1/public/system/info`   | Get platform branding, name, logo |
| GET    | `/api/v1/public/system/status` | Health check with DB/Redis status |

### Tenants

| Method | Endpoint                        | Description                                       |
| ------ | ------------------------------- | ------------------------------------------------- |
| GET    | `/api/v1/public/tenants/{slug}` | Get tenant info by slug (for login page branding) |

### Themes

| Method | Endpoint                | Description            |
| ------ | ----------------------- | ---------------------- |
| GET    | `/api/v1/public/themes` | List all public themes |

### i18n

| Method | Endpoint                                    | Description                 |
| ------ | ------------------------------------------- | --------------------------- |
| GET    | `/api/v1/public/i18n/locales`               | List available locales      |
| GET    | `/api/v1/public/i18n/translations/{locale}` | Get translations for locale |

---

## Auth Endpoints

### Authentication Flow

| Method | Endpoint               | Description          | Body                                                                        |
| ------ | ---------------------- | -------------------- | --------------------------------------------------------------------------- |
| POST   | `/api/v1/auth/login`   | Login and get tokens | `{ "username": "email", "password": "..." }`                                |
| POST   | `/api/v1/auth/refresh` | Refresh access token | `{ "refresh_token": "..." }`                                                |
| POST   | `/api/v1/auth/logout`  | Invalidate session   | -                                                                           |
| POST   | `/api/v1/auth/signup`  | Register new tenant  | `{ "email", "password", "full_name", "org_name", "org_slug", "plan_type" }` |

### Setup (First-time)

| Method | Endpoint                                 | Description                |
| ------ | ---------------------------------------- | -------------------------- |
| GET    | `/api/v1/auth/setup/status`              | Check if setup is complete |
| POST   | `/api/v1/auth/setup`                     | Initial system setup       |
| GET    | `/api/v1/auth/setup/check-slug?slug=xxx` | Check slug availability    |
| POST   | `/api/v1/auth/setup/verify-key`          | Verify setup key           |

### Sessions (Requires: Authentication)

| Method | Endpoint                                      | Permission    | Description               |
| ------ | --------------------------------------------- | ------------- | ------------------------- |
| GET    | `/api/v1/auth/sessions`                       | (self)        | List own sessions         |
| DELETE | `/api/v1/auth/sessions/{sid}`                 | (self)        | Revoke own session        |
| GET    | `/api/v1/auth/admin/sessions`                 | `user:read`   | List all sessions (admin) |
| DELETE | `/api/v1/auth/admin/sessions/{user_id}/{sid}` | `user:update` | Force logout user         |

---

## Me Endpoints (User Context)

| Method | Endpoint                 | Description                                |
| ------ | ------------------------ | ------------------------------------------ |
| GET    | `/api/v1/me/preferences` | Get user preferences (theme, locale)       |
| PATCH  | `/api/v1/me/preferences` | Update preferences                         |
| GET    | `/api/v1/me/menus`       | Get aggregated menu items (core + plugins) |

**Example Response: `/api/v1/me/preferences`**

```json
{
  "theme_mode": "dark",
  "locale": "th",
  "sidebar_collapsed": false,
  "accent_color": "#6366f1"
}
```

---

## Admin Endpoints

### Config & Settings

| Method | Endpoint                 | Permission             | Description                |
| ------ | ------------------------ | ---------------------- | -------------------------- |
| GET    | `/api/v1/admin/config`   | `system:config:read`   | Get system config          |
| PATCH  | `/api/v1/admin/config`   | `system:config:update` | Update config              |
| GET    | `/api/v1/admin/settings` | `system:config:read`   | Get settings with metadata |
| PATCH  | `/api/v1/admin/settings` | `system:config:update` | Update settings            |

### Users

| Method | Endpoint                                  | Permission            | Description            |
| ------ | ----------------------------------------- | --------------------- | ---------------------- |
| GET    | `/api/v1/admin/users`                     | `user:read`           | List users (paginated) |
| GET    | `/api/v1/admin/users/{id}`                | `user:read`           | Get user by ID         |
| PATCH  | `/api/v1/admin/users/{id}`                | `user:update`         | Update user            |
| DELETE | `/api/v1/admin/users/{id}`                | `user:delete`         | Delete user            |
| POST   | `/api/v1/admin/users/{id}/reset-password` | `user:reset_password` | Force password reset   |

**Query Params:** `?page=1&limit=20&search=xxx&sort=created_at`

### Tenants

| Method | Endpoint                                   | Permission      | Description                |
| ------ | ------------------------------------------ | --------------- | -------------------------- |
| GET    | `/api/v1/admin/tenants`                    | `tenant:read`   | List tenants               |
| GET    | `/api/v1/admin/tenants/me`                 | (self)          | Get own tenant             |
| GET    | `/api/v1/admin/tenants/{id}`               | `tenant:read`   | Get tenant by ID           |
| PATCH  | `/api/v1/admin/tenants/owner`              | `tenant:update` | Update own tenant settings |
| POST   | `/api/v1/admin/tenants/{id}/verify-domain` | `tenant:update` | Verify custom domain       |

### Roles & Permissions (RBAC)

| Method | Endpoint                               | Permission          | Description             |
| ------ | -------------------------------------- | ------------------- | ----------------------- |
| GET    | `/api/v1/admin/roles`                  | `role:read`         | List roles              |
| POST   | `/api/v1/admin/roles`                  | `role:create`       | Create role             |
| PATCH  | `/api/v1/admin/roles/{id}`             | `role:update`       | Update role             |
| DELETE | `/api/v1/admin/roles/{id}`             | `role:delete`       | Delete role             |
| GET    | `/api/v1/admin/roles/{id}/permissions` | `role:read`         | Get role permissions    |
| PATCH  | `/api/v1/admin/roles/{id}/permissions` | `role:update`       | Update role permissions |
| GET    | `/api/v1/admin/permissions`            | `permission:read`   | List all permissions    |
| PATCH  | `/api/v1/admin/permissions/{id}`       | `permission:update` | Update permission       |

### Plugins ⭐ NEW GRANULAR PERMISSIONS

| Method | Endpoint                                 | Permission         | Description                    |
| ------ | ---------------------------------------- | ------------------ | ------------------------------ |
| GET    | `/api/v1/admin/plugins`                  | `plugin:read`      | List installed plugins         |
| GET    | `/api/v1/admin/plugins/{id}`             | `plugin:read`      | Get plugin details             |
| POST   | `/api/v1/admin/plugins`                  | `plugin:install`   | Install plugin from manifest   |
| POST   | `/api/v1/admin/plugins/analyze`          | `plugin:install`   | Analyze plugin security        |
| POST   | `/api/v1/admin/plugins/install-approved` | `plugin:approve`   | Install with security approval |
| DELETE | `/api/v1/admin/plugins/{id}`             | `plugin:uninstall` | Uninstall plugin               |
| POST   | `/api/v1/admin/plugins/{id}/enable`      | `plugin:enable`    | Enable plugin                  |
| POST   | `/api/v1/admin/plugins/{id}/disable`     | `plugin:enable`    | Disable plugin                 |
| PUT    | `/api/v1/admin/plugins/{id}/config`      | `plugin:configure` | Update plugin config           |
| GET    | `/api/v1/admin/plugins/{id}/security`    | `plugin:read`      | Get security analysis          |

### Themes ⭐ NEW GRANULAR PERMISSIONS

| Method | Endpoint                               | Permission       | Description              |
| ------ | -------------------------------------- | ---------------- | ------------------------ |
| POST   | `/api/v1/admin/themes/import`          | `theme:import`   | Import theme (multipart) |
| PATCH  | `/api/v1/admin/themes/{id}/activate`   | `theme:activate` | Set active theme         |
| PATCH  | `/api/v1/admin/themes/{id}/visibility` | `theme:update`   | Update visibility        |
| DELETE | `/api/v1/admin/themes/{id}`            | `theme:delete`   | Delete theme             |

### i18n ⭐ NEW GRANULAR PERMISSIONS

| Method | Endpoint                            | Permission    | Description           |
| ------ | ----------------------------------- | ------------- | --------------------- |
| GET    | `/api/v1/admin/i18n/locales`        | `i18n:read`   | List locales          |
| POST   | `/api/v1/admin/i18n/locales`        | `i18n:manage` | Create locale         |
| DELETE | `/api/v1/admin/i18n/locales/{code}` | `i18n:manage` | Delete locale         |
| GET    | `/api/v1/admin/i18n/translations`   | `i18n:read`   | List all translations |
| PATCH  | `/api/v1/admin/i18n/translations`   | `i18n:manage` | Update translation    |
| POST   | `/api/v1/admin/i18n/keys`           | `i18n:manage` | Create key            |
| DELETE | `/api/v1/admin/i18n/keys/{key}`     | `i18n:manage` | Delete key            |

### Audit Logs

| Method | Endpoint             | Permission          | Description     |
| ------ | -------------------- | ------------------- | --------------- |
| GET    | `/api/v1/admin/logs` | `system:audit:read` | List audit logs |

**Query Params:** `?page=1&limit=50&user_id=xxx&action=xxx`

### API Keys

| Method | Endpoint                      | Permission              | Description    |
| ------ | ----------------------------- | ----------------------- | -------------- |
| GET    | `/api/v1/admin/api-keys`      | `system:api_key:manage` | List API keys  |
| POST   | `/api/v1/admin/api-keys`      | `system:api_key:manage` | Create API key |
| DELETE | `/api/v1/admin/api-keys/{id}` | `system:api_key:manage` | Revoke API key |

### CORS

| Method | Endpoint                  | Permission           | Description          |
| ------ | ------------------------- | -------------------- | -------------------- |
| GET    | `/api/v1/admin/cors`      | `system:cors:manage` | List allowed origins |
| POST   | `/api/v1/admin/cors`      | `system:cors:manage` | Add origin           |
| DELETE | `/api/v1/admin/cors/{id}` | `system:cors:manage` | Remove origin        |

### CMS Pages ⭐ NEW

| Method | Endpoint                       | Permission  | Description    |
| ------ | ------------------------------ | ----------- | -------------- |
| GET    | `/api/v1/admin/cms/pages`      | `cms:read`  | List CMS pages |
| GET    | `/api/v1/admin/cms/pages/{id}` | `cms:read`  | Get page by ID |
| POST   | `/api/v1/admin/cms/pages`      | `cms:write` | Create page    |
| PUT    | `/api/v1/admin/cms/pages/{id}` | `cms:write` | Update page    |
| DELETE | `/api/v1/admin/cms/pages/{id}` | `cms:write` | Delete page    |

### Resource Sharing ⭐ NEW (RULE 19)

| Method | Endpoint                          | Permission     | Description          |
| ------ | --------------------------------- | -------------- | -------------------- |
| GET    | `/api/v1/admin/shares`            | `share:read`   | List created shares  |
| GET    | `/api/v1/admin/shares/received`   | `share:read`   | List received shares |
| POST   | `/api/v1/admin/shares`            | `share:create` | Create share         |
| DELETE | `/api/v1/admin/shares/{id}`       | `share:delete` | Revoke share         |
| GET    | `/api/v1/admin/shares/{id}/usage` | `share:read`   | Get usage count      |

**Share Request Body**:

```json
{
  "resource_type": "role|theme|page|media",
  "resource_id": "uuid",
  "shared_to_tenant_id": "uuid",
  "can_reshare": true
}
```

### Branding Management

| Method | Endpoint                           | Permission        | Description              |
| ------ | ---------------------------------- | ----------------- | ------------------------ |
| GET    | `/api/v1/admin/branding`           | `branding:read`   | Get current branding     |
| PATCH  | `/api/v1/admin/branding`           | `branding:update` | Update branding settings |
| GET    | `/api/v1/admin/branding/contexts`  | `branding:read`   | List all contexts        |
| GET    | `/api/v1/admin/branding/{context}` | `branding:read`   | Get by context           |
| PATCH  | `/api/v1/admin/branding/{context}` | `branding:update` | Update context branding  |

**Contexts**: `console`, `workspace`

---

## Workspace Endpoints

### Workspace Context

| Method | Endpoint                       | Permission    | Description           |
| ------ | ------------------------------ | ------------- | --------------------- |
| GET    | `/api/v1/workspace/status`     | (public)      | Get workspace status  |
| GET    | `/api/v1/workspace/roles`      | `role:read`   | List workspace roles  |
| POST   | `/api/v1/workspace/roles`      | `role:create` | Create workspace role |
| PATCH  | `/api/v1/workspace/roles/{id}` | `role:update` | Update workspace role |
| DELETE | `/api/v1/workspace/roles/{id}` | `role:delete` | Delete workspace role |

---

## Media Endpoints

| Method | Endpoint                               | Permission     | Description             |
| ------ | -------------------------------------- | -------------- | ----------------------- |
| POST   | `/api/v1/media`                        | `media:upload` | Upload file (multipart) |
| GET    | `/api/v1/media`                        | `media:read`   | List assets             |
| GET    | `/api/v1/media/assets`                 | `media:read`   | List assets (alternate) |
| GET    | `/api/v1/media/assets/{id}`            | `media:read`   | Get asset by ID         |
| DELETE | `/api/v1/media/assets/{id}`            | `media:delete` | Delete asset            |
| POST   | `/api/v1/media/folders`                | `media:folder` | Create folder           |
| GET    | `/api/v1/media/{tenant_id}/{filename}` | (public)       | Serve file              |

---

## Signal Endpoints (kyx-signal Integration)

> **Note:** Signal endpoints are used for WebSocket connection to kyx-signal realtime server.

### Token Exchange

| Method | Endpoint                | Permission  | Description                                     |
| ------ | ----------------------- | ----------- | ----------------------------------------------- |
| GET    | `/api/v1/signal/health` | (public)    | Health check for signal integration             |
| POST   | `/api/v1/signal/token`  | `chat:read` | Generate signal ticket for WebSocket connection |

### Request: `/api/v1/signal/token`

```json
{
  "rooms": ["room-uuid-1", "room-uuid-2"], // Optional: restrict to rooms
  "ttl": 60 // Optional: TTL in seconds (10-300)
}
```

### Response

```json
{
  "ticket": "eyJhbGciOiJIUzI1NiIsInR5cCI6...",
  "expires_at": "2026-01-07T03:05:00Z",
  "signal_url": "wss://signal.kyx.io/ws"
}
```

### Permission Mapping

| Kernel Permission | Signal Permission | Description                |
| ----------------- | ----------------- | -------------------------- |
| `chat:read`       | `subscribe`       | Subscribe to room messages |
| `chat:send`       | `publish`         | Publish messages to room   |
| `room:join`       | `join`            | Join/leave rooms           |
| `room:admin`      | `moderate`        | Kick/ban/mute users        |

### Rate Limits by Plan

| Plan       | Messages/min | Connections | Rooms |
| ---------- | ------------ | ----------- | ----- |
| Free       | 30           | 2           | 5     |
| Pro        | 200          | 10          | 50    |
| Enterprise | 1000         | 100         | 500   |

---

## Permission Reference

### User Permissions

| Permission            | Description          |
| --------------------- | -------------------- |
| `user:read`           | View user list       |
| `user:create`         | Create users         |
| `user:update`         | Update users         |
| `user:delete`         | Delete users         |
| `user:reset_password` | Force password reset |

### Role & Permission

| Permission                             | Description        |
| -------------------------------------- | ------------------ |
| `role:read/create/update/delete`       | Manage roles       |
| `permission:read/create/update/delete` | Manage permissions |

### Plugin Permissions ⭐ NEW

| Permission         | Description                      |
| ------------------ | -------------------------------- |
| `plugin:read`      | View plugins                     |
| `plugin:install`   | Install new plugins              |
| `plugin:enable`    | Enable/disable plugins           |
| `plugin:uninstall` | Uninstall plugins                |
| `plugin:configure` | Update plugin config             |
| `plugin:approve`   | Approve security-flagged plugins |

### Theme Permissions ⭐ NEW

| Permission       | Description           |
| ---------------- | --------------------- |
| `theme:import`   | Import themes         |
| `theme:activate` | Activate themes       |
| `theme:update`   | Update theme settings |
| `theme:delete`   | Delete themes         |

### i18n Permissions ⭐ NEW

| Permission    | Description                 |
| ------------- | --------------------------- |
| `i18n:read`   | View locales/translations   |
| `i18n:manage` | Manage locales/translations |

### System Permissions

| Permission              | Description          |
| ----------------------- | -------------------- |
| `system:config:read`    | View system config   |
| `system:config:update`  | Update system config |
| `system:api_key:manage` | Manage API keys      |
| `system:cors:manage`    | Manage CORS origins  |
| `system:audit:read`     | View audit logs      |

### Media Permissions

| Permission     | Description    |
| -------------- | -------------- |
| `media:read`   | View assets    |
| `media:upload` | Upload files   |
| `media:delete` | Delete assets  |
| `media:folder` | Manage folders |

### CMS Permissions ⭐ NEW

| Permission    | Description       |
| ------------- | ----------------- |
| `cms:read`    | View CMS pages    |
| `cms:write`   | Create/edit pages |
| `cms:publish` | Publish pages     |
| `cms:delete`  | Delete pages      |

### Sharing Permissions ⭐ NEW

| Permission     | Description   |
| -------------- | ------------- |
| `share:read`   | View shares   |
| `share:create` | Create shares |
| `share:delete` | Revoke shares |

### Branding Permissions ⭐ NEW

| Permission        | Description            |
| ----------------- | ---------------------- |
| `branding:read`   | View branding settings |
| `branding:update` | Update branding        |

### Tenant Permissions

| Permission                         | Description    |
| ---------------------------------- | -------------- |
| `tenant:read/create/update/delete` | Manage tenants |

---

## Common Response Formats

### Success

```json
{
  "status": "success",
  "message": "Operation completed"
}
```

### Error

```json
{
  "error": "Error type",
  "message": "Detailed error message"
}
```

### Paginated List

```json
{
  "data": [...],
  "total": 100,
  "page": 1,
  "limit": 20
}
```

---

## Frontend Integration Example

```typescript
// lib/api.ts
const API_BASE = "http://localhost:8080";

export async function fetchWithAuth(
  endpoint: string,
  options: RequestInit = {},
) {
  const token = localStorage.getItem("access_token");
  return fetch(`${API_BASE}${endpoint}`, {
    ...options,
    headers: {
      "Content-Type": "application/json",
      Authorization: `Bearer ${token}`,
      ...options.headers,
    },
  });
}

// Example: Get users
const users = await fetchWithAuth("/api/v1/admin/users?page=1&limit=20").then(
  (r) => r.json(),
);

// Example: Update preferences
await fetchWithAuth("/api/v1/me/preferences", {
  method: "PATCH",
  body: JSON.stringify({ theme_mode: "dark" }),
});
```

---

## OpenAPI Documentation

- **Swagger UI:** http://localhost:8080/api/v1/docs
- **ReDoc:** http://localhost:8080/api/v1/redoc
- **OpenAPI JSON:** http://localhost:8080/api-doc/openapi.json

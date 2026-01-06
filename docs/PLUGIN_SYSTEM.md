# Plugin System Documentation

## Overview

The Kyx Kernel Plugin System provides a secure, capability-based architecture for extending platform functionality through external modules.

## Architecture

```
┌─────────────────────────────────────────────────────────┐
│                     Plugin API                          │
│  /api/v1/admin/plugins/*                               │
├─────────────────────────────────────────────────────────┤
│                  PluginRegistry                         │
│  - DashMap cache for loaded plugins                     │
│  - Lifecycle management (install/enable/disable)        │
├─────────────────────────────────────────────────────────┤
│                   Host Functions                        │
│  - PluginKvStore (storage)                             │
│  - PluginEventBus (events)                             │
│  - Logging (info/warn/error)                           │
├─────────────────────────────────────────────────────────┤
│                   WASM Engine                           │
│  - wasmer runtime                                       │
│  - Sandboxed execution                                  │
├─────────────────────────────────────────────────────────┤
│                  Database Layer                         │
│  - sys_plugins table                                    │
│  - sys_plugin_events audit log                          │
└─────────────────────────────────────────────────────────┘
```

## API Endpoints

| Method | Endpoint                                 | Description            |
| ------ | ---------------------------------------- | ---------------------- |
| GET    | `/api/v1/admin/plugins`                  | List installed plugins |
| POST   | `/api/v1/admin/plugins`                  | Install plugin         |
| POST   | `/api/v1/admin/plugins/analyze`          | Security analysis      |
| POST   | `/api/v1/admin/plugins/install-approved` | Install with approval  |
| GET    | `/api/v1/admin/plugins/{id}`             | Get plugin details     |
| DELETE | `/api/v1/admin/plugins/{id}`             | Uninstall plugin       |
| POST   | `/api/v1/admin/plugins/{id}/enable`      | Enable plugin          |
| POST   | `/api/v1/admin/plugins/{id}/disable`     | Disable plugin         |
| PUT    | `/api/v1/admin/plugins/{id}/config`      | Update config          |
| GET    | `/api/v1/admin/plugins/{id}/security`    | Security warnings      |

## Capabilities

Plugins declare required capabilities in their manifest:

| Capability            | Risk Level | Description               |
| --------------------- | ---------- | ------------------------- |
| `storage_read`        | Low        | Read from plugin KV store |
| `storage_write`       | Low        | Write to plugin KV store  |
| `event_emit`          | Low        | Emit events               |
| `event_subscribe`     | Low        | Subscribe to events       |
| `log_info/warn/error` | Low        | Logging                   |
| `ui_register_page`    | Medium     | Register UI pages         |
| `camera/microphone`   | Medium     | Device access             |
| `http_request`        | Medium     | Outbound HTTP             |
| `user_profile_write`  | High       | Modify user data          |
| `financial_read`      | High       | Read financial data       |
| `financial_write`     | Critical   | Modify financial data     |
| `tenant_data_write`   | Critical   | Modify tenant data        |

## Security

### Risk Levels

- **Low**: Safe, auto-approved
- **Medium**: Reviewed, logged
- **High**: Requires explicit approval
- **Critical**: Requires admin approval with audit trail

### Approval Workflow

1. Install request with dangerous capabilities → blocked
2. Admin uses `/install-approved` endpoint
3. Must provide `approved_by` (admin UUID) and `approval_reason`
4. Full audit trail logged in `sys_plugin_events`

## Manifest Format

Compatible with pixco-customer-app external plugins:

```json
{
  "id": "my-plugin",
  "name": "My Plugin",
  "version": "1.0.0",
  "description": "Plugin description",
  "entry": "./index.js",
  "author": {
    "name": "Developer",
    "email": "dev@example.com",
    "website": "https://example.com"
  },
  "icon": "🔌",
  "category": "Utilities",
  "tags": ["example"],
  "permissions": ["camera"],
  "dataAccess": ["user_profile"],
  "networkAccess": ["api.example.com"],
  "runtime": "frontend",
  "capabilities": ["storage_read", "event_emit"],
  "minAppVersion": "2.0.0",
  "verified": true,
  "official": false,
  "featured": false
}
```

## Database Schema

### sys_plugins

- `id` - UUID primary key
- `tenant_id` - Tenant ownership
- `plugin_id` - Unique plugin identifier
- `name`, `version`, `description`
- `author`, `author_email`, `author_website`
- `icon`, `banner`, `category`, `tags`
- `runtime` - frontend/wasm/service/hybrid
- `capabilities`, `permissions`, `data_access`, `network_access`
- `verified`, `official`, `featured`, `is_core_plugin`
- `status` - installed/enabled/disabled/error/pending_approval
- `wasm_path`, `wasm_hash`, `wasm_size_bytes`

### sys_plugin_events

- `id` - UUID primary key
- `plugin_id` - FK to sys_plugins
- `tenant_id` - FK to auth_tenants
- `event_type` - installed/enabled/disabled/etc
- `event_data` - JSON audit data
- `created_at` - Timestamp

## Permissions

| Code  | Slug             | Description       |
| ----- | ---------------- | ----------------- |
| PLG.R | plugin:read      | View plugins      |
| PLG.I | plugin:install   | Install plugins   |
| PLG.E | plugin:enable    | Enable/disable    |
| PLG.U | plugin:uninstall | Remove plugins    |
| PLG.C | plugin:configure | Modify settings   |
| PLG.A | plugin:approve   | Approve dangerous |

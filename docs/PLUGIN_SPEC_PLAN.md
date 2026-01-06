# Kyx Plugin Specification: Plan Plugin

> **Plugin ID:** `kyx-plan`  
> **Version:** 1.0.0  
> **Status:** Draft  
> **Date:** 2026-01-07

---

## 1. Overview

Optional billing/subscription plugin for tenant plan management. System works without this plugin (no limits applied).

### Dependencies

- **Required:** None (fully independent)
- **Optional:** None

---

## 2. Features

| Feature              | Description                            |
| -------------------- | -------------------------------------- |
| **Custom Plans**     | Tenant-specific plan definitions       |
| **Plan Templates**   | Global templates (free/pro/enterprise) |
| **Feature Flags**    | Per-plan feature toggles               |
| **Quota Management** | Usage limits per resource type         |
| **Plan Inheritance** | Extend from templates                  |

---

## 3. Database Schema

```sql
-- Plans table (can be global or tenant-specific)
CREATE TABLE plugin_plans (
  id UUID PRIMARY KEY,
  tenant_id UUID REFERENCES sys_tenants(id) NULL,  -- NULL = global template
  name VARCHAR(100) NOT NULL,
  display_name VARCHAR(200),
  description TEXT,
  is_template BOOLEAN DEFAULT false,
  parent_plan_id UUID REFERENCES plugin_plans(id) NULL,
  features JSONB DEFAULT '{}',
  quotas JSONB DEFAULT '{}',
  is_active BOOLEAN DEFAULT true,
  created_at TIMESTAMPTZ DEFAULT NOW(),
  updated_at TIMESTAMPTZ DEFAULT NOW()
);

-- User plan assignments
CREATE TABLE plugin_user_plans (
  id UUID PRIMARY KEY,
  user_id UUID NOT NULL,
  tenant_id UUID NOT NULL,
  plan_id UUID REFERENCES plugin_plans(id),
  started_at TIMESTAMPTZ DEFAULT NOW(),
  expires_at TIMESTAMPTZ NULL,
  is_active BOOLEAN DEFAULT true
);
```

---

## 4. Features & Quotas Schema

```json
{
  "features": {
    "max_connections": 10,
    "messages_per_minute": 200,
    "max_rooms": 50,
    "ai_enabled": true,
    "custom_domain": true,
    "white_label": false,
    "priority_support": false
  },
  "quotas": {
    "storage_mb": 5000,
    "api_calls_per_day": 10000,
    "users_per_tenant": 100,
    "integrations": 10
  }
}
```

---

## 5. API Endpoints

| Method | Endpoint                          | Permission    | Description           |
| ------ | --------------------------------- | ------------- | --------------------- |
| GET    | `/api/v1/plans`                   | (public)      | List available plans  |
| GET    | `/api/v1/plans/templates`         | (public)      | List global templates |
| POST   | `/api/v1/admin/plans`             | `plan:create` | Create custom plan    |
| PATCH  | `/api/v1/admin/plans/{id}`        | `plan:update` | Update plan           |
| DELETE | `/api/v1/admin/plans/{id}`        | `plan:delete` | Delete plan           |
| POST   | `/api/v1/admin/plans/{id}/assign` | `plan:assign` | Assign plan to user   |
| GET    | `/api/v1/me/plan`                 | (self)        | Get current plan      |

---

## 6. Plugin Hooks

```rust
// Called when plugin loads
fn on_activate(kernel: &Kernel) -> Result<()>;

// Called on every request (for quota checking)
fn on_request(claims: &Claims) -> Option<PlanContext>;

// Called for rate limiting
fn get_quota(user_id: Uuid, resource: &str) -> Option<u64>;
```

---

## 7. Graceful Degradation

| Scenario             | Behavior                        |
| -------------------- | ------------------------------- |
| Plugin not installed | No limits, all features enabled |
| Plan expired         | Falls back to "free" template   |
| DB error             | Uses cached plan or no limits   |

---

## 8. Default Templates (Seeded)

| Name       | Connections | Msg/min | AI  | Domain |
| ---------- | ----------- | ------- | --- | ------ |
| free       | 2           | 30      | ❌  | ❌     |
| pro        | 10          | 200     | ✅  | ✅     |
| enterprise | 100         | 1000    | ✅  | ✅     |

---

## 9. Manifest

```yaml
id: kyx-plan
name: "Kyx Plan Manager"
version: "1.0.0"
author: "Kyx Team"
description: "Subscription and plan management"
capabilities:
  - database:read
  - database:write
  - claims:extend
permissions:
  - plan:read
  - plan:create
  - plan:update
  - plan:delete
  - plan:assign
settings:
  default_plan: "free"
  cache_ttl_seconds: 300
```

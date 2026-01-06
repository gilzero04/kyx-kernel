# Kyx Plugin Specification: Signal Plugin

> **Plugin ID:** `kyx-signal`  
> **Version:** 1.0.0  
> **Status:** Draft  
> **Date:** 2026-01-07

---

## 1. Overview

Realtime communication plugin providing WebSocket connections, notifications, and presence. Connects to external kyx-signal service.

### Dependencies

- **Required:** External kyx-signal service
- **Optional:** `kyx-plan` (for connection quotas)

---

## 2. Features

| Feature                  | Description               |
| ------------------------ | ------------------------- |
| **WebSocket Proxy**      | Token-based WS connection |
| **Push Notifications**   | FCM/APNS delivery         |
| **In-App Notifications** | Realtime alerts           |
| **Presence**             | Online/offline status     |
| **Typing Indicators**    | Real-time typing events   |
| **Event Dispatch**       | Pub/sub messaging         |

---

## 3. Architecture

```
┌─────────────────────────────────────────────────────────┐
│                     KYX-KERNEL                          │
│  ┌─────────────────────────────────────────────────────┐│
│  │            kyx-signal-plugin (thin client)          ││
│  │  - Token issuance                                   ││
│  │  - API proxy to kyx-signal                          ││
│  └─────────────────────┬───────────────────────────────┘│
└────────────────────────┼────────────────────────────────┘
                         │ HTTP/WS
                         ▼
┌─────────────────────────────────────────────────────────┐
│                    KYX-SIGNAL SERVICE                   │
│  PushService | EventDispatcher | SessionManager        │
│  WebSocket connections, FCM/APNS, Presence             │
└─────────────────────────────────────────────────────────┘
```

---

## 4. Database Schema

```sql
-- Notification preferences (in Kernel DB)
CREATE TABLE plugin_notification_prefs (
  id UUID PRIMARY KEY,
  user_id UUID NOT NULL UNIQUE,
  email_enabled BOOLEAN DEFAULT true,
  push_enabled BOOLEAN DEFAULT true,
  in_app_enabled BOOLEAN DEFAULT true,
  quiet_hours_start TIME,
  quiet_hours_end TIME,
  created_at TIMESTAMPTZ DEFAULT NOW(),
  updated_at TIMESTAMPTZ DEFAULT NOW()
);

-- Notification log (for audit/retry)
CREATE TABLE plugin_notifications (
  id UUID PRIMARY KEY,
  user_id UUID NOT NULL,
  tenant_id UUID NOT NULL,
  type VARCHAR(50) NOT NULL,
  title VARCHAR(200),
  body TEXT,
  data JSONB,
  channel VARCHAR(20),  -- push, websocket, email
  status VARCHAR(20) DEFAULT 'pending',
  sent_at TIMESTAMPTZ,
  error TEXT,
  created_at TIMESTAMPTZ DEFAULT NOW()
);
```

---

## 5. API Endpoints

| Method               | Endpoint                         | Permission               | Description         |
| -------------------- | -------------------------------- | ------------------------ | ------------------- |
| **Token Exchange**   |
| POST                 | `/api/v1/signal/token`           | `signal:connect`         | Get signal ticket   |
| GET                  | `/api/v1/signal/health`          | (public)                 | Health check        |
| **Notifications**    |
| POST                 | `/api/v1/signal/notify`          | `notification:send`      | Send notification   |
| POST                 | `/api/v1/signal/broadcast`       | `notification:broadcast` | Broadcast to tenant |
| **User Preferences** |
| GET                  | `/api/v1/me/notifications/prefs` | (self)                   | Get preferences     |
| PATCH                | `/api/v1/me/notifications/prefs` | (self)                   | Update preferences  |
| **Push Tokens**      |
| POST                 | `/api/v1/me/push-token`          | (self)                   | Register push token |
| DELETE               | `/api/v1/me/push-token/{device}` | (self)                   | Unregister device   |

---

## 6. Notification Types

```yaml
notification_types:
  system:
    - system.announcement
    - system.maintenance
  user:
    - user.welcome
    - user.password_changed
  social:
    - message.received
    - mention.user
  payment:
    - payment.received
    - subscription.expiring
```

---

## 7. Plugin Hooks

```rust
// Send notification (called by core or other plugins)
fn notify(user_id: Uuid, notification: Notification) -> Result<()>;

// Broadcast to tenant
fn broadcast(tenant_id: Uuid, notification: Notification) -> Result<()>;

// Check if user is online (presence)
fn is_online(user_id: Uuid) -> bool;

// Get connection count (for rate limiting)
fn connection_count(user_id: Uuid) -> u32;
```

---

## 8. Integration with kyx-plan (Optional)

```rust
// If kyx-plan is installed, apply connection limits
fn on_websocket_connect(user_id: Uuid) -> Result<()> {
    if let Some(plan) = plugins.get("kyx-plan") {
        let features = plan.get_features(user_id);
        let current = connection_count(user_id);

        if current >= features.max_connections {
            return Err("Connection limit reached");
        }
    }
    // No plan plugin = unlimited connections
    Ok(())
}
```

---

## 9. Graceful Degradation

| Scenario               | Behavior                          |
| ---------------------- | --------------------------------- |
| Plugin not installed   | No realtime features, use polling |
| kyx-signal unavailable | Queue notifications, retry later  |
| kyx-plan not installed | Unlimited connections             |

---

## 10. Manifest

```yaml
id: kyx-signal
name: "Kyx Realtime & Notifications"
version: "1.0.0"
author: "Kyx Team"
description: "WebSocket, push notifications, presence"
capabilities:
  - database:read
  - database:write
  - http:external
  - websocket:proxy
permissions:
  - signal:connect
  - notification:send
  - notification:broadcast
settings:
  signal_url: "${SIGNAL_WS_URL}"
  default_ticket_ttl: 60
  retry_attempts: 3
optional_integrations:
  - kyx-plan # For connection quotas
```

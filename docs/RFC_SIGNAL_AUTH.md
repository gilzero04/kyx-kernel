# RFC: Kyx Signal Authentication Protocol

**RFC ID:** KYX-RFC-001  
**Status:** DRAFT  
**Author:** Kyx Architecture Team  
**Date:** 2026-01-07  
**Version:** 1.0

---

## 1. Abstract

This document specifies the authentication protocol between **kyx-kernel** (Authorization Server) and **kyx-signal** (Realtime Server). It defines a secure token exchange mechanism that enables WebSocket connections without exposing long-lived credentials.

---

## 2. Motivation

### Problems with Direct JWT

- Long-lived access tokens in WebSocket URLs = security risk
- WebSocket connections cannot rotate tokens mid-session
- Audit complexity when token is used across services

### Solution: Signal Ticket Model

- Short-lived, single-use tickets
- Service-specific scope
- Auditable exchange flow

---

## 3. Terminology

| Term                | Definition                                          |
| ------------------- | --------------------------------------------------- |
| **Kernel Token**    | Standard JWT issued by kyx-kernel (access_token)    |
| **Signal Ticket**   | Short-lived JWT for kyx-signal connection           |
| **Ticket Exchange** | Process of converting kernel token to signal ticket |

---

## 4. Token Structure

### 4.1 Kernel Token (Existing)

```json
{
  "sub": "user-uuid",
  "role": "superadmin",
  "tenant_id": "tenant-uuid",
  "permissions": ["chat:read", "chat:send", "room:join"],
  "is_system_owner": true,
  "sid": "session-uuid",
  "exp": 1704600000,
  "token_type": "access"
}
```

### 4.2 Signal Ticket (New)

```json
{
  "sub": "user-uuid",
  "tenant_id": "tenant-uuid",
  "signal_permissions": ["subscribe", "publish", "join"],
  "allowed_rooms": ["*"],
  "plan": "pro",
  "rate_limits": {
    "messages_per_minute": 100,
    "connections": 5
  },
  "iat": 1704599940,
  "exp": 1704600000,
  "jti": "ticket-uuid",
  "iss": "kyx-kernel",
  "aud": "kyx-signal",
  "token_type": "signal_ticket"
}
```

### 4.3 Claim Definitions

| Claim                | Type      | Required | Description                       |
| -------------------- | --------- | -------- | --------------------------------- |
| `sub`                | UUID      | ✅       | User ID                           |
| `tenant_id`          | UUID      | ✅       | Tenant ID                         |
| `signal_permissions` | Array     | ✅       | Mapped permissions for signal     |
| `allowed_rooms`      | Array     | ❌       | Room restrictions (["*"] = all)   |
| `plan`               | String    | ✅       | Plan type (free/pro/enterprise)   |
| `rate_limits`        | Object    | ✅       | Rate limit config                 |
| `jti`                | UUID      | ✅       | Unique ticket ID (for single-use) |
| `iss`                | String    | ✅       | Issuer (always "kyx-kernel")      |
| `aud`                | String    | ✅       | Audience (always "kyx-signal")    |
| `exp`                | Timestamp | ✅       | Expiration (30-60 seconds)        |

---

## 5. Permission Mapping

### 5.1 Kernel → Signal Mapping Table

| Kernel Permission | Signal Permission | Description                |
| ----------------- | ----------------- | -------------------------- |
| `chat:read`       | `subscribe`       | Subscribe to room messages |
| `chat:send`       | `publish`         | Publish messages to room   |
| `room:join`       | `join`            | Join/leave rooms           |
| `room:admin`      | `moderate`        | Kick/ban/mute users        |
| `room:create`     | `create_room`     | Create new rooms           |

### 5.2 Mapping Logic

```rust
fn map_permissions(kernel_perms: &[String]) -> Vec<String> {
    let mapping = [
        ("chat:read", "subscribe"),
        ("chat:send", "publish"),
        ("room:join", "join"),
        ("room:admin", "moderate"),
        ("room:create", "create_room"),
    ];

    kernel_perms.iter()
        .filter_map(|p| mapping.iter()
            .find(|(k, _)| k == p)
            .map(|(_, s)| s.to_string()))
        .collect()
}
```

---

## 6. Token Exchange Flow

### 6.1 Sequence Diagram

```
┌────────┐          ┌─────────────┐          ┌────────────┐
│ Client │          │ kyx-kernel  │          │ kyx-signal │
└───┬────┘          └──────┬──────┘          └─────┬──────┘
    │                      │                       │
    │ 1. Login             │                       │
    ├─────────────────────►│                       │
    │                      │                       │
    │ 2. Kernel Token      │                       │
    │◄─────────────────────┤                       │
    │                      │                       │
    │ 3. Request Signal    │                       │
    │    Ticket            │                       │
    ├─────────────────────►│                       │
    │                      │                       │
    │ 4. Signal Ticket     │                       │
    │◄─────────────────────┤                       │
    │                      │                       │
    │ 5. WebSocket Connect │                       │
    │    ?ticket=xxx       │                       │
    ├──────────────────────┼──────────────────────►│
    │                      │                       │
    │                      │  6. Verify Ticket     │
    │                      │     (Local JWT)       │
    │                      │                       │
    │ 7. Connection OK     │                       │
    │◄─────────────────────┼───────────────────────┤
    │                      │                       │
```

### 6.2 Step Details

| Step | Actor  | Action                    | Notes                           |
| ---- | ------ | ------------------------- | ------------------------------- |
| 1    | Client | POST /api/v1/auth/login   | Standard login                  |
| 2    | Kernel | Return access_token       | Long-lived (15m-1h)             |
| 3    | Client | POST /api/v1/signal/token | With Authorization header       |
| 4    | Kernel | Return signal_ticket      | Short-lived (30-60s)            |
| 5    | Client | WS /ws?ticket=xxx         | Connect to signal               |
| 6    | Signal | Verify JWT signature      | Local verification, no callback |
| 7    | Signal | Connection established    | Start session                   |

---

## 7. API Specification

### 7.1 Request Signal Ticket

**Endpoint:** `POST /api/v1/signal/token`

**Headers:**

```
Authorization: Bearer <kernel_access_token>
Content-Type: application/json
```

**Request Body (Optional):**

```json
{
  "rooms": ["room-1", "room-2"],
  "ttl": 60
}
```

**Response (200 OK):**

```json
{
  "ticket": "eyJhbGciOiJIUzI1NiIsInR5cCI6IkpXVCJ9...",
  "expires_at": "2026-01-07T02:50:00Z",
  "signal_url": "wss://signal.kyx.io/ws"
}
```

**Error Responses:**

| Status | Error          | Description                     |
| ------ | -------------- | ------------------------------- |
| 401    | `unauthorized` | Invalid or expired kernel token |
| 403    | `forbidden`    | No signal permissions           |
| 429    | `rate_limited` | Too many ticket requests        |

### 7.2 WebSocket Connection

**URL:** `wss://signal.kyx.io/ws?ticket=<signal_ticket>`

**Alternative (Header):**

```
Authorization: Bearer <signal_ticket>
Upgrade: websocket
```

---

## 8. Security Considerations

### 8.1 Ticket Properties

- **Single-use:** Each ticket should only be used once
- **Short-lived:** 30-60 second expiration
- **Scoped:** Contains only signal-relevant claims
- **Bound:** Contains user ID and tenant ID for audit

### 8.2 Key Management

- Signal and Kernel share signing key
- Key rotation via key ID (kid) in JWT header
- Keys stored in secure vault (not env vars in production)

### 8.3 Rate Limiting

- Ticket requests: 10/minute per user
- Ticket requests: 100/minute per tenant
- WebSocket connections: Per-plan limit

---

## 9. Implementation Checklist

### Kernel Side

- [ ] Create `/api/v1/signal/token` endpoint
- [ ] Implement permission mapping
- [ ] Implement ticket generation
- [ ] Add rate limiting
- [ ] Add OpenAPI documentation

### Signal Side

- [ ] Implement ticket verification
- [ ] Extract claims from ticket
- [ ] Initialize connection with claims
- [ ] Handle ticket expiration (reconnect flow)

---

## 10. Future Considerations

- **Ticket revocation:** Redis-based revocation list
- **Room-specific tickets:** For room invitations
- **Delegated tickets:** For service-to-service calls
- **MQTT support:** Same ticket model, different transport

---

## Appendix A: Example Implementation

### Rust: Generate Signal Ticket

```rust
pub struct SignalTicketClaims {
    pub sub: Uuid,
    pub tenant_id: Uuid,
    pub signal_permissions: Vec<String>,
    pub allowed_rooms: Vec<String>,
    pub plan: String,
    pub rate_limits: RateLimits,
    pub jti: Uuid,
    pub iss: String,
    pub aud: String,
    pub exp: i64,
    pub iat: i64,
    pub token_type: String,
}

pub fn generate_signal_ticket(
    user: &User,
    kernel_permissions: &[String],
    plan: &Plan,
) -> Result<String> {
    let now = chrono::Utc::now().timestamp();
    let claims = SignalTicketClaims {
        sub: user.id,
        tenant_id: user.tenant_id,
        signal_permissions: map_permissions(kernel_permissions),
        allowed_rooms: vec!["*".to_string()],
        plan: plan.name.clone(),
        rate_limits: plan.rate_limits.clone(),
        jti: Uuid::new_v4(),
        iss: "kyx-kernel".to_string(),
        aud: "kyx-signal".to_string(),
        exp: now + 60, // 60 seconds
        iat: now,
        token_type: "signal_ticket".to_string(),
    };

    encode(&Header::default(), &claims, &EncodingKey::from_secret(secret))
}
```

### TypeScript: Client Usage

```typescript
async function connectToSignal() {
  // 1. Get signal ticket
  const res = await fetch("/api/v1/signal/token", {
    method: "POST",
    headers: {
      Authorization: `Bearer ${accessToken}`,
      "Content-Type": "application/json",
    },
  });
  const { ticket, signal_url } = await res.json();

  // 2. Connect WebSocket
  const ws = new WebSocket(`${signal_url}?ticket=${ticket}`);

  // 3. Handle reconnection
  ws.onclose = () => {
    setTimeout(connectToSignal, 1000);
  };
}
```

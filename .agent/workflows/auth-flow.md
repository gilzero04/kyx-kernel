# Authentication Flow
// turbo-all

## JWT Structure

### Access Token (short-lived: 30 min)
```json
{
  "sub": "user-uuid",
  "email": "user@example.com",
  "role": "superadmin",
  "tenant_id": "tenant-uuid",  // Root Tenant ID for Root Admin, Child ID for others
  "scope": "global|tenant",
  "permissions": ["user:read", "user:create", "tenant:read"],
  "exp": 1703001234
}
```

### Refresh Token (long-lived: 24 hours)
```json
{
  "sub": "user-uuid",
  "type": "refresh",
  "exp": 1703087634
}
```

## Flow Diagrams

### Login Flow
```
Client → POST /auth/login (email, password)
       ← { access_token, refresh_token, user }
```

### Token Refresh
```
Client → POST /auth/refresh { refresh_token }
       ← { access_token, refresh_token }
```

### Logout
```
Client → POST /auth/logout (Authorization: Bearer token)
       ← { status: "success" }
```

## Scoped Access

| User Type | JWT tenant_id | Visibility |
|-----------|---------------|------------|
| **System Owner** | `root_uuid` | Full system branch (unfiltered) |
| **Tenant Admin** | `<uuid>` | Own tenant + descendants |
| **Branch Admin** | `<uuid>` | Own branch only |

## Setup Flow (First-Time)

```
1. Client → POST /auth/setup/verify-key (X-Engine-Secret)
         ← 200 OK or 401 Unauthorized

2. Client → POST /auth/register (X-Engine-Secret)
         ← { access_token, refresh_token, user, tenant }
```

## Token Storage (Frontend)

```typescript
// Access token: memory only (authStore)
// Refresh token: localStorage or httpOnly cookie
localStorage.setItem('refresh_token', token);
```

# Kyx Platform — Production Setup Guide

> **Version**: 1.0.0  
> **Date**: 2026-01-18  
> **For**: Production Deployment

---

## Table of Contents

1. [Prerequisites](#prerequisites)
2. [Initial Setup](#initial-setup)
3. [First-Time Configuration](#first-time-configuration)
4. [Frontend Integration](#frontend-integration)
5. [Security Hardening](#security-hardening)
6. [Testing](#testing)
7. [Troubleshooting](#troubleshooting)

---

## Prerequisites

### System Requirements

- Docker 24.0+
- Docker Compose 2.0+
- PostgreSQL 15+
- Redis 7.0+
- Node.js 20+ (for frontend)

### Ports

- `8080`: Kyx Kernel API
- `5432`: PostgreSQL
- `6379`: Redis
- `5173`: Frontend (dev)

---

## Initial Setup

### 1. Clone Repository

```bash
git clone https://github.com/kyx-tech/kyx-kernel.git
cd kyx-kernel
```

### 2. Environment Variables

Create `.env` file:

```bash
# Database
DATABASE_URL=postgresql://kyx:kyx123@localhost:5432/kyx_db

# Redis
REDIS_URL=redis://localhost:6379

# JWT
JWT_SECRET=your-super-secret-jwt-key-change-in-production
JWT_EXPIRY_SECONDS=604800

# CORS
ALLOWED_ORIGINS=http://localhost:5173,https://yourdomain.com

# Server
RUST_LOG=info
PORT=8080
HOST=0.0.0.0

# Storage
UPLOAD_DIR=./uploads
MAX_UPLOAD_SIZE_MB=10

# Optional: Engine Key for first-time setup
ENGINE_KEY=your-engine-key-here
```

### 3. Start Services

```bash
# Start PostgreSQL + Redis
docker compose up -d postgres redis

# Run migrations
docker compose run --rm kyx-kernel.sqlx database setup
docker compose run --rm kyx-kernel sqlx migrate run

# Start Kernel
docker compose up -d kyx-kernel
```

### 4. Verify Installation

```bash
curl http://localhost:8080/api/v1/public/system/status
```

**Expected Response**:

```json
{
  "status": "healthy",
  "database": "connected",
  "redis": "connected"
}
```

---

## First-Time Configuration

### Step 1: Check Setup Status

```bash
curl http://localhost:8080/api/v1/auth/setup/status
```

**Response**:

```json
{
  "is_initialized": false,
  "requires_engine_key": true
}
```

### Step 2: Verify Engine Key (if required)

```bash
curl -X POST http://localhost:8080/api/v1/auth/setup/verify-key \
  -H "Content-Type: application/json" \
  -d '{"engine_key":"your-engine-key"}'
```

### Step 3: Check Slug Availability

```bash
curl "http://localhost:8080/api/v1/auth/setup/check-slug?slug=my-company"
```

**Response**:

```json
{
  "available": true
}
```

### Step 4: Initialize System

```bash
curl -X POST http://localhost:8080/api/v1/auth/setup \
  -H "Content-Type: application/json" \
  -d '{
    "engine_key": "your-engine-key",
    "org_name": "My Company",
    "org_slug": "my-company",
    "admin_email": "admin@company.com",
    "admin_password": "SecurePass123!",
    "admin_name": "Admin User"
  }'
```

**Response**:

```json
{
  "success": true,
  "message": "System initialized",
  "data": {
    "tenant_id": "uuid-here",
    "user_id": "uuid-here",
    "token": "eyJhbGciOiJIUzI1NiIs..."
  }
}
```

**Save the token!** You'll need it for authentication.

---

## Frontend Integration

### 1. Install Frontend

```bash
cd path/to/kyx-platform
bun install
```

### 2. Configure Environment

Create `.env.local`:

```bash
PUBLIC_API_BASE_URL=http://localhost:8080/api/v1
PUBLIC_WS_URL=ws://localhost:8080/ws
VITE_USE_LOCAL_THEMES=true
```

### 3. Start Development Server

```bash
bun run dev
```

### 4. Login

Navigate to `http://localhost:5173/console` and login with:

- Email: `admin@company.com`
- Password: `SecurePass123!`

---

## Frontend Authentication Flow

### TypeScript Example

```typescript
// lib/api/client.ts
const API_BASE = import.meta.env.PUBLIC_API_BASE_URL;

export async function login(email: string, password: string) {
  const response = await fetch(`${API_BASE}/auth/login`, {
    method: "POST",
    headers: { "Content-Type": "application/json" },
    body: JSON.stringify({ email, password }),
  });

  const data = await response.json();

  if (data.success) {
    localStorage.setItem("access_token", data.data.token);
    return data.data.user;
  }

  throw new Error(data.message);
}

export async function fetchWithAuth(
  endpoint: string,
  options: RequestInit = {},
) {
  const token = localStorage.getItem("access_token");

  const response = await fetch(`${API_BASE}${endpoint}`, {
    ...options,
    headers: {
      "Content-Type": "application/json",
      Authorization: `Bearer ${token}`,
      ...options.headers,
    },
  });

  if (response.status === 401) {
    // Token expired, redirect to login
    localStorage.removeItem("access_token");
    window.location.href = "/login";
    throw new Error("Unauthorized");
  }

  return response.json();
}

// Usage
const users = await fetchWithAuth("/admin/users?page=1&limit=20");
```

---

## Security Hardening

### 1. Change Default Credentials

```bash
# Update admin password via API
curl -X POST http://localhost:8080/api/v1/admin/users/{user_id}/reset-password \
  -H "Authorization: Bearer YOUR_TOKEN" \
  -H "Content-Type: application/json" \
  -d '{"new_password":"NewSecurePass456!"}'
```

### 2. JWT Secret Rotation

```bash
# Generate new secret
openssl rand -base64 64

# Update .env
JWT_SECRET=newly-generated-secret

# Restart kernel
docker compose restart kyx-kernel
```

### 3. CORS Configuration

```bash
# Add allowed origin
curl -X POST http://localhost:8080/api/v1/admin/cors \
  -H "Authorization: Bearer YOUR_TOKEN" \
  -H "Content-Type: application/json" \
  -d '{"origin":"https://yourdomain.com"}'
```

### 4. Rate Limiting (per-tenant)

Edit `.env`:

```bash
RATE_LIMIT_REQUESTS_PER_MINUTE=60
RATE_LIMIT_BURST=10
```

### 5. Enable HTTPS

Use reverse proxy (Nginx/Caddy):

**Caddy Example**:

```caddyfile
api.yourdomain.com {
    reverse_proxy localhost:8080
}
```

---

## Testing

### 1. Run E2E Tests

```bash
# Start services
docker compose up -d

# Wait for services
sleep 5

# Run endpoint tests
./scripts/test-endpoints.sh http://localhost:8080
```

**Expected Output**:

```
╔═══════════════════════════════════════════════════════════╗
║  PASS: 27  |  FAIL: 0  |  SKIP: 1  |  TOTAL: 28         ║
╚═══════════════════════════════════════════════════════════╝
```

### 2. Run Permission Matrix Tests

```bash
cargo test --test permission_matrix_tests
```

**Expected Output**:

```
test result: ok. 13 passed; 0 failed; 0 ignored
```

### 3. Frontend Verification

```bash
# Open browser
open http://localhost:5173/console

# Expected:
# ✅ Console Dashboard loads
# ✅ Login works
# ✅ Theme switching works
# ✅ No 401/403 errors
```

---

## Troubleshooting

### Database Connection Failed

```bash
# Check PostgreSQL
docker compose logs postgres

# Test connection
psql postgresql://kyx:kyx123@localhost:5432/kyx_db -c "SELECT 1"
```

### Redis Connection Failed

```bash
# Check Redis
docker compose logs redis

# Test connection
redis-cli ping
```

### JWT Token Invalid

```bash
# Verify JWT_SECRET matches in .env
# Re-login to get new token
curl -X POST http://localhost:8080/api/v1/auth/login \
  -d '{"email":"admin@company.com","password":"your-password"}'
```

### CORS Error

```bash
# Add origin to whitelist
curl -X POST http://localhost:8080/api/v1/admin/cors \
  -H "Authorization: Bearer YOUR_TOKEN" \
  -d '{"origin":"http://localhost:5173"}'
```

### Theme Not Loading (Frontend)

```bash
# Check VITE_USE_LOCAL_THEMES in .env.local
VITE_USE_LOCAL_THEMES=true

# Restart dev server
bun run dev
```

### Permission Denied

```bash
# Check user roles
curl http://localhost:8080/api/v1/auth/me \
  -H "Authorization: Bearer YOUR_TOKEN"

# Expected: roles array should contain "admin" or "superadmin"
```

---

## Production Checklist

### Before Deployment

- [ ] Change all default passwords
- [ ] Generate new JWT_SECRET
- [ ] Configure CORS whitelist
- [ ] Setup HTTPS/TLS
- [ ] Configure backups (database)
- [ ] Setup monitoring (logs, metrics)
- [ ] Test all critical endpoints
- [ ] Run permission matrix tests
- [ ] Verify frontend integration

### After Deployment

- [ ] Monitor error logs
- [ ] Check database connections
- [ ] Verify Redis cache hits
- [ ] Test authentication flow
- [ ] Confirm CORS configuration
- [ ] Monitor API response times
- [ ] Setup alerting

---

## Next Steps

1. **Create Additional Users**: [User Management API](API_REFERENCE.md#user-management)
2. **Setup Roles & Permissions**: [RBAC Guide](API_REFERENCE.md#rbac-roles--permissions)
3. **Configure Themes**: [Theme Management](API_REFERENCE.md#theme-management)
4. **Install Plugins**: [Plugin System](PLUGIN_CONTRACT_V1.md)

---

**Need Help?**

- Documentation: `/docs`
- API Reference: `API_REFERENCE.md`
- GitHub Issues: https://github.com/kyx-tech/kyx-kernel/issues

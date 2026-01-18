---
description: How to run complete backend testing from fresh deploy to E2E tests
---

# Backend Testing Workflow

Complete testing workflow for Kyx Kernel following BACKEND_TESTING_FLOW.md phases.

## Prerequisites

- Docker and Docker Compose installed
- kyx-infra and kyx-kernel repositories available
- Terminal access

---

## Quick Start (Full Test)

// turbo-all

```bash
cd /Users/beykyu/Developer/kyx-tech/kyx-kernel
./scripts/deploy-fresh.sh
```

This runs Pre-Phase + Phase 0 automatically.

---

## Phase-by-Phase Testing

### Pre-Phase: Fresh Installation

// turbo

1. Clean postgres volume:

```bash
cd /Users/beykyu/Developer/kyx-tech/kyx-infra/docker/postgres && docker compose down -v
```

// turbo 2. Clean redis volume:

```bash
cd /Users/beykyu/Developer/kyx-tech/kyx-infra/docker/redis && docker compose down -v
```

// turbo 3. Start postgres:

```bash
cd /Users/beykyu/Developer/kyx-tech/kyx-infra/docker/postgres && docker compose up -d
```

// turbo 4. Start redis:

```bash
cd /Users/beykyu/Developer/kyx-tech/kyx-infra/docker/redis && docker compose up -d
```

// turbo 5. Wait for infra:

```bash
sleep 15
```

// turbo 6. Clean kernel:

```bash
cd /Users/beykyu/Developer/kyx-tech/kyx-kernel && docker compose down -v
```

7. Build kernel (takes 10-15 minutes):

```bash
cd /Users/beykyu/Developer/kyx-tech/kyx-kernel && docker compose build --no-cache
```

// turbo 8. Start kernel:

```bash
cd /Users/beykyu/Developer/kyx-tech/kyx-kernel && docker compose up -d && sleep 20
```

---

### Phase 0: System Initialization

// turbo

1. Check setup status:

```bash
curl -s http://localhost:8080/api/v1/auth/setup/status | jq .
```

Expected: `{"is_setup": false}`

2. Initialize system (read ENGINE_SECRET from .env):

```bash
ENGINE_SECRET=$(grep ENGINE_SECRET_KEY /Users/beykyu/Developer/kyx-tech/kyx-kernel/.env | cut -d= -f2)
curl -s -X POST http://localhost:8080/api/v1/auth/setup \
  -H "Content-Type: application/json" \
  -H "X-Engine-Secret: $ENGINE_SECRET" \
  -d '{
    "org_name": "KYZ Technologied Co., LTD.",
    "org_slug": "kyz-tech",
    "email": "admin@kyz.tech",
    "password": "Admin123!",
    "full_name": "Admin User",
    "platform_type": "multi"
  }' | jq .
```

// turbo 3. Save token:

```bash
TOKEN=$(curl -s -X POST http://localhost:8080/api/v1/auth/setup \
  -H "Content-Type: application/json" \
  -H "X-Engine-Secret: $(grep ENGINE_SECRET_KEY /Users/beykyu/Developer/kyx-tech/kyx-kernel/.env | cut -d= -f2)" \
  -d '{"org_name":"KYZ","org_slug":"kyz","email":"admin@kyz.tech","password":"Admin123!","full_name":"Admin","platform_type":"multi"}' \
  | jq -r '.access_token')
echo $TOKEN > /tmp/kyx_token.txt
echo "Token saved to /tmp/kyx_token.txt"
```

---

### Phase 1: Health Check

// turbo

1. Check backend health:

```bash
curl -s http://localhost:8080/health | jq .
```

// turbo 2. Check system info:

```bash
curl -s http://localhost:8080/api/v1/public/system/info | jq .
```

---

### Phase 2: Authentication

// turbo

1. Login with saved credentials:

```bash
curl -s -X POST http://localhost:8080/api/v1/auth/login \
  -H "Content-Type: application/json" \
  -d '{"username":"admin@kyz.tech","password":"Admin123!"}' | jq .
```

// turbo 2. Verify token (use saved token):

```bash
TOKEN=$(cat /tmp/kyx_token.txt)
curl -s http://localhost:8080/api/v1/auth/me \
  -H "Authorization: Bearer $TOKEN" | jq .
```

---

### Phase 3: RBAC Verification

// turbo

1. List roles:

```bash
TOKEN=$(cat /tmp/kyx_token.txt)
curl -s http://localhost:8080/api/v1/admin/roles \
  -H "Authorization: Bearer $TOKEN" | jq .
```

// turbo 2. List permissions:

```bash
TOKEN=$(cat /tmp/kyx_token.txt)
curl -s http://localhost:8080/api/v1/admin/permissions \
  -H "Authorization: Bearer $TOKEN" | jq .
```

---

### Phase 4: Business Logic

// turbo

1. List users:

```bash
TOKEN=$(cat /tmp/kyx_token.txt)
curl -s http://localhost:8080/api/v1/admin/users \
  -H "Authorization: Bearer $TOKEN" | jq .
```

// turbo 2. List tenants:

```bash
TOKEN=$(cat /tmp/kyx_token.txt)
curl -s http://localhost:8080/api/v1/admin/tenants \
  -H "Authorization: Bearer $TOKEN" | jq .
```

// turbo 3. List themes:

```bash
TOKEN=$(cat /tmp/kyx_token.txt)
curl -s http://localhost:8080/api/v1/admin/themes \
  -H "Authorization: Bearer $TOKEN" | jq .
```

---

### Phase 5: E2E Integration Tests

// turbo
Run full E2E test suite:

```bash
cd /Users/beykyu/Developer/kyx-tech/kyx-kernel && ./scripts/test-endpoints.sh http://localhost:8080
```

Expected: `PASS: 59, FAIL: 0, SKIP: 0`

---

### Phase 6: Security & Performance

// turbo

1. Test rate limiting (should get 429 after 60 requests):

```bash
for i in {1..65}; do
  STATUS=$(curl -s -o /dev/null -w "%{http_code}" http://localhost:8080/health)
  echo "Request $i: $STATUS"
done
```

// turbo 2. Test CORS headers:

```bash
curl -s -I -X OPTIONS http://localhost:8080/api/v1/auth/login \
  -H "Origin: http://localhost:5173" \
  -H "Access-Control-Request-Method: POST"
```

---

## Scripts Reference

| Script              | Purpose                     |
| ------------------- | --------------------------- |
| `deploy-fresh.sh`   | Full fresh deploy + Phase 0 |
| `test-endpoints.sh` | E2E API tests (Phase 5)     |

---

## Credentials

| Field    | Value              |
| -------- | ------------------ |
| Email    | admin@kyz.tech     |
| Password | Admin123!          |
| Token    | /tmp/kyx_token.txt |

---

## Documentation

- [BACKEND_TESTING_FLOW.md](file:///Users/beykyu/Developer/kyx-tech/kyx-kernel/docs/BACKEND_TESTING_FLOW.md) - Full testing guide
- [API_REFERENCE.md](file:///Users/beykyu/Developer/kyx-tech/kyx-kernel/docs/API_REFERENCE.md) - API documentation

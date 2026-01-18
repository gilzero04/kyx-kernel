# Backend Testing Flow — Complete Guide

> **Version**: 1.1  
> **Purpose**: Step-by-step testing flow for backend API validation  
> **Date**: 2026-01-18

---

## 📋 Overview

การทดสอบระบบหลังบ้านแบ่งเป็น **7 ระดับ (Phase 0-6)**:

0. **System Initialization** — Setup ระบบครั้งแรก (ถ้า `is_setup: false`)
1. **System Health Check** — ตรวจสอบระบบพื้นฐาน
2. **Authentication Flow** — ทดสอบการ login/session
3. **RBAC Verification** — ตรวจสอบ permissions
4. **Business Logic** — ทดสอบ CRUD operations
5. **Integration Tests** — E2E scenarios
6. **Security & Performance** — Rate limit, CORS, Response time

---

## Pre-Phase: Fresh Installation (เริ่มต้นใหม่ทั้งหมด) 🔄

> **ทำเมื่อ**: ต้องการเริ่มต้นระบบใหม่หมด (ลบข้อมูลเดิมทั้งหมด)

### Step 1: Clean Up Existing Infrastructure

```bash
# ไปที่ kyx-infra directory
cd /Users/beykyu/Developer/kyx-tech/kyx-infra

# Stop and remove all containers + volumes
docker compose down -v
```

**Expected**:

```
✅ Container kyx-kernel Removed
✅ Container postgres Removed
✅ Container redis Removed
✅ Volume kyx-infra_postgres_data Removed
✅ Volume kyx-infra_redis_data Removed
```

### Step 2: Rebuild Containers

```bash
# Rebuild all images
docker compose build
```

**Expected**:

```
✅ Building kyx-kernel...
✅ Successfully built
```

### Step 3: Start Services

```bash
# Start all services in detached mode
docker compose up -d

# Wait 10 seconds for services to initialize
sleep 10
```

**Expected**:

```
✅ Container postgres Started
✅ Container redis Started
✅ Container kyx-kernel Started
```

### Step 4: Verify Services Running

```bash
# Check service status
docker compose ps

# Check logs
docker compose logs kyx-kernel --tail=20
```

**Expected**:

```
NAME         STATUS
postgres     Up
redis        Up
kyx-kernel   Up (healthy)
```

### Step 5: Run Database Migrations

```bash
# Run migrations (if needed)
docker compose run --rm kyx-kernel sqlx migrate run
```

**Expected**:

```
✅ Applied migrations
✅ Database schema up to date
```

### Step 6: Verify Backend Health

```bash
# Check health endpoint
curl http://localhost:8080/health

# Check system status
curl http://localhost:8080/api/v1/public/system/info | jq .
```

**Expected**:

- Health: `200 OK`
- System info returned with version

### Step 7: Check Setup Status

```bash
curl -s http://localhost:8080/api/v1/auth/setup/status | jq .
```

**Expected**:

```json
{
  "is_setup": false // ✅ พร้อม initialize
}
```

**✅ ระบบพร้อมแล้ว! ไปต่อที่ Phase 0 เพื่อ initialize**

---

## Phase 0: System Initialization (ถ้ายังไม่ได้ setup) 🔧

> **ทำเฉพาะเมื่อ** `"is_setup": false`

### 🔍 Setup มี 2 แบบ:

| Mode            | Use Case                   | Engine Key    | Security |
| --------------- | -------------------------- | ------------- | -------- |
| **Development** | Local development, Testing | ❌ ไม่ต้องใช้ | Low      |
| **Production**  | Production deployment      | ✅ ต้องใช้    | High     |

#### Development Mode (ไม่ใช้ Engine Key)

- **เมื่อ**: `ENGINE_SECRET_KEY` ไม่ได้ตั้งค่าใน `.env`
- **ความปลอดภัย**: ต่ำ - ใครก็ initialize ได้
- **ใช้กับ**: Local development, Docker Compose, Testing

#### Production Mode (ใช้ Engine Key)

- **เมื่อ**: `ENGINE_SECRET_KEY` ตั้งค่าแล้วใน `.env`
- **ความปลอดภัย**: สูง - ต้องมี secret key จึงจะ initialize ได้
- **ใช้กับ**: Production, Staging, Public-facing deployments

---

### Setup Endpoints (4 endpoints):

| Endpoint                 | Method | Purpose                      | Auth            |
| ------------------------ | ------ | ---------------------------- | --------------- |
| `/auth/setup/status`     | GET    | ตรวจสอบว่า setup แล้วหรือยัง | ❌ Public       |
| `/auth/setup/verify-key` | POST   | ทดสอบ engine key             | ✅ Engine Key   |
| `/auth/setup/check-slug` | GET    | ตรวจสอบ slug available       | ⚠️ Optional Key |
| `/auth/setup`            | POST   | Initialize ระบบ              | ⚠️ Optional Key |

---

### 0.1 ตรวจสอบ Setup Status ก่อน

```bash
curl -s http://localhost:8080/api/v1/auth/setup/status | jq .
```

**Expected**:

```json
{
  "is_setup": false // ❌ ยังไม่ได้ setup
}
```

### 0.2 ตรวจสอบ Slug Availability

#### Development Mode (ไม่ใช้ Engine Key):

```bash
curl -s "http://localhost:8080/api/v1/auth/setup/check-slug?slug=kyz-tech" | jq .
```

#### Production Mode (ใช้ Engine Key):

```bash
curl -s "http://localhost:8080/api/v1/auth/setup/check-slug?slug=kyz-tech" \
  -H "X-Engine-Secret: your-production-secret-key" | jq .
```

**Expected**:

```json
{
  "available": true
}
```

### 0.3 Initialize System

#### Development Mode (ไม่ใช้ Engine Key):

```bash
curl -s -X POST http://localhost:8080/api/v1/auth/setup \
  -H "Content-Type: application/json" \
  -d '{
    "org_name": "KYZ Technologied Co., LTD.",
    "org_slug": "kyz-tech",
    "admin_email": "admin@kyz.tech",
    "admin_password": "SecurePass123!",
    "admin_name": "Admin User"
  }' | jq .
```

#### Production Mode (ใช้ Engine Key):

```bash
# ตั้งค่า ENGINE_SECRET_KEY ใน .env ก่อน
# ENGINE_SECRET_KEY=your-super-secret-production-key

curl -s -X POST http://localhost:8080/api/v1/auth/setup \
  -H "Content-Type: application/json" \
  -H "X-Engine-Secret: your-super-secret-production-key" \
  -d '{
    "org_name": "KYZ Technologied Co., LTD.",
    "org_slug": "kyz-tech",
    "admin_email": "admin@kyz.tech",
    "admin_password": "SecurePass123!",
    "admin_name": "Admin User"
  }' | jq .
```

**Expected**:

```json
{
  "success": true,
  "data": {
    "tenant_id": "uuid-here",
    "user_id": "uuid-here",
    "token": "jwt-token-here"
  }
}
```

**📝 สิ่งที่ต้องจดบันทึก**:

- ✅ `admin_email` & `admin_password` → ใช้ login ใน Phase 2
- ✅ `token` → ใช้ทดสอบ API ได้ทันที
- ✅ `tenant_id` → Root tenant ของระบบ

### 0.4 Verify Setup Complete

```bash
curl -s http://localhost:8080/api/v1/auth/setup/status | jq .
```

**Expected**:

```json
{
  "is_setup": true // ✅ เปลี่ยนเป็น true แล้ว
}
```

**✅ พร้อมแล้ว! ไปต่อที่ Phase 1**

### 0.5 ตรวจสอบสิ่งที่ได้จาก Setup ✅

#### ✅ Root Tenant ถูกสร้าง

```bash
curl -s http://localhost:8080/api/v1/admin/tenants \
  -H "Authorization: Bearer $TOKEN" | jq '.data.items[] | {id, name, slug, tenant_type}'
```

**Expected**:

```json
{
  "id": "uuid-here",
  "name": "KYZ Technologied Co., LTD.",
  "slug": "kyz-tech",
  "tenant_type": "owner"
}
```

#### ✅ Admin User ถูกสร้าง

```bash
curl -s http://localhost:8080/api/v1/admin/users \
  -H "Authorization: Bearer $TOKEN" | jq '.data.items[] | {email, full_name, role}'
```

**Expected**:

```json
{
  "email": "admin@kyz.tech",
  "full_name": "Admin User",
  "role": "superadmin"
}
```

#### ✅ Branding Records สร้างแล้ว (2 contexts)

```bash
# Get tenant branding_id
TENANT_BRANDING_ID=$(curl -s http://localhost:8080/api/v1/admin/tenants/me \
  -H "Authorization: Bearer $TOKEN" | jq -r '.branding_id')

echo "Branding ID: $TENANT_BRANDING_ID"

# Verify branding exists in database
psql postgresql://kyx:kyx123@localhost:5432/kyx_db \
  -c "SELECT context, name FROM sys_brandings WHERE tenant_id = (SELECT id FROM auth_tenants WHERE slug='kyz-tech')"
```

**Expected**:

```
  context  |    name
-----------+------------
 console   | KYZ Technologied Co., LTD.
 workspace | KYZ Technologied Co., LTD.
```

#### ✅ Default Themes ถูก Seed

```bash
curl -s http://localhost:8080/api/v1/public/themes | jq '.[] | {id, name, slug, type}'
```

**Expected** (อย่างน้อย 2 themes):

```json
{
  "id": "uuid",
  "name": "Kyx Dark",
  "slug": "kyx-dark",
  "type": "dark"
}
{
  "id": "uuid",
  "name": "Kyx Light",
  "slug": "kyx-light",
  "type": "light"
}
```

#### ✅ Default Permissions ถูก Seed

```bash
curl -s http://localhost:8080/api/v1/admin/permissions \
  -H "Authorization: Bearer $TOKEN" | jq '.data | length'
```

**Expected**: `40+` permissions

#### ✅ Default Roles ถูกสร้าง

```bash
curl -s http://localhost:8080/api/v1/admin/roles \
  -H "Authorization: Bearer $TOKEN" | jq '.data.items[] | {name, code}'
```

**Expected** (อย่างน้อย):

```json
{"name": "Super Admin", "code": "superadmin"}
{"name": "Admin", "code": "admin"}
{"name": "Manager", "code": "manager"}
```

**สรุป Setup**: ระบบสร้าง Root Tenant, Admin User, 2 Branding Records, Default Themes, Permissions, และ Roles ✅

---

## Phase 1: System Health Check ✅

> **ทำเมื่อ** `"is_setup": true`

### 1.1 ตรวจสอบ Services

```bash
# Database
docker ps | grep postgres
psql postgresql://kyx:kyx123@localhost:5432/kyx_db -c "SELECT 1"

# Redis
docker ps | grep redis
redis-cli ping

# Backend
curl http://localhost:8080/health
```

**Expected**:

- ✅ PostgreSQL: Running
- ✅ Redis: PONG
- ✅ Backend: 200 OK

### 1.2 ตรวจสอบ System Info

```bash
curl -s http://localhost:8080/api/v1/public/system/info | jq .
```

**Expected**:

```json
{
  "name": "Kyx Kernel",
  "version": "1.0.0",
  "environment": "local"
}
```

---

## Phase 2: Authentication Flow ✅

### 2.1 Login (Get Token)

```bash
LOGIN_RESPONSE=$(curl -s -X POST http://localhost:8080/api/v1/auth/login \
  -H "Content-Type: application/json" \
  -d '{
    "username": "admin@kyx.tech",
    "password": "Admin123!"
  }')

echo $LOGIN_RESPONSE | jq .

# Extract token
TOKEN=$(echo $LOGIN_RESPONSE | jq -r '.access_token')
echo "Token: $TOKEN"
```

**Expected**:

- ✅ `access_token`: JWT string
- ✅ `user.email`: "admin@kyx.tech"
- ✅ `user.role`: "superadmin"
- ✅ `user.permissions`: Array with 40+ permissions

### 2.2 Verify Token

```bash
curl -s http://localhost:8080/api/v1/auth/me \
  -H "Authorization: Bearer $TOKEN" | jq .
```

**Expected**:

- ✅ 200 OK
- ✅ User profile returned

### 2.3 Test Invalid Token

```bash
curl -s http://localhost:8080/api/v1/admin/users \
  -H "Authorization: Bearer invalid_token"
```

**Expected**:

- ✅ 401 Unauthorized

---

## Phase 3: RBAC Verification ✅

### 3.1 List Roles

```bash
curl -s http://localhost:8080/api/v1/admin/roles \
  -H "Authorization: Bearer $TOKEN" | jq .
```

**Expected**:

- ✅ 200 OK
- ✅ Contains `superadmin` role
- ✅ Pagination info

### 3.2 List Permissions

```bash
curl -s http://localhost:8080/api/v1/admin/permissions \
  -H "Authorization: Bearer $TOKEN" | jq .
```

**Expected**:

- ✅ 200 OK
- ✅ Contains system permissions
- ✅ Permissions have `code`, `name`, `description`

### 3.3 Test Permission Matrix

```bash
cd /Users/beykyu/Developer/kyx-tech/kyx-kernel
cargo test --test permission_matrix_tests
```

**Expected**:

```
test result: ok. 13 passed; 0 failed; 0 ignored
```

---

## Phase 4: Business Logic Tests ✅

### 4.1 User Management

#### List Users

```bash
curl -s "http://localhost:8080/api/v1/admin/users?page=1&limit=20" \
  -H "Authorization: Bearer $TOKEN" | jq .
```

**Expected**:

- ✅ 200 OK
- ✅ `data.items`: Array of users
- ✅ `data.pagination`: page info

### 4.2 Tenant Management

#### List Tenants

```bash
curl -s http://localhost:8080/api/v1/admin/tenants \
  -H "Authorization: Bearer $TOKEN" | jq .
```

**Expected**:

- ✅ 200 OK
- ✅ Contains root tenant

#### Get Current Tenant

```bash
curl -s http://localhost:8080/api/v1/admin/tenants/me \
  -H "Authorization: Bearer $TOKEN" | jq .
```

**Expected**:

- ✅ 200 OK
- ✅ Tenant info with `id`, `name`, `slug`

### 4.3 Theme Management

#### List Themes

```bash
curl -s http://localhost:8080/api/v1/public/themes | jq .
```

**Expected**:

- ✅ 200 OK
- ✅ Contains `kyx-dark`, `kyx-light`

### 4.4 Media Library

#### List Media

```bash
curl -s http://localhost:8080/api/v1/media \
  -H "Authorization: Bearer $TOKEN" | jq .
```

**Expected**:

- ✅ 200 OK
- ✅ Empty array or list of assets

### 4.5 CMS Pages

#### List Pages

```bash
curl -s http://localhost:8080/api/v1/admin/cms/pages \
  -H "Authorization: Bearer $TOKEN" | jq .
```

**Expected**:

- ✅ 200 OK
- ✅ Empty or list of pages

---

## Phase 5: Integration Tests (E2E) ✅

### 5.1 Run Full Endpoint Tests

```bash
cd /Users/beykyu/Developer/kyx-tech/kyx-kernel
./scripts/test-endpoints.sh http://localhost:8080
```

**Expected**:

```
╔═══════════════════════════════════════════════════════════╗
║  PASS: 59  |  FAIL: 0  |  SKIP: 0  |  TOTAL: 59         ║
╚═══════════════════════════════════════════════════════════╝
All tests passed!
```

### 5.2 Cross-Context Validation

#### Console → Workspace (Should PASS)

```bash
# Console user accessing tenant data
curl -s http://localhost:8080/api/v1/admin/tenants \
  -H "Authorization: Bearer $TOKEN" | jq .
```

**Expected**: ✅ 200 OK

---

## Phase 6: Security & Performance Tests 🔐

### 6.1 Rate Limiting Tests

#### Test Rate Limit (60 requests/minute default)

```bash
# Rapid fire 70 requests
for i in {1..70}; do
  STATUS=$(curl -s -o /dev/null -w "%{http_code}" http://localhost:8080/api/v1/public/system/info)
  echo "Request $i: $STATUS"
done
```

**Expected**:

- First 60 requests: `200 OK`
- Requests 61+: `429 Too Many Requests`

#### Verify Rate Limit Headers

```bash
curl -v http://localhost:8080/api/v1/public/system/info 2>&1 | grep -i "x-ratelimit"
```

**Expected Headers**:

```
X-RateLimit-Limit: 60
X-RateLimit-Remaining: 59
X-RateLimit-Reset: 1705567200
```

### 6.2 Token Expiration Test

#### Test Expired Token

```bash
# Get token
TOKEN=$(curl -s -X POST http://localhost:8080/api/v1/auth/login \
  -H "Content-Type: application/json" \
  -d '{"username":"admin@kyx.tech","password":"Admin123!"}' \
  | jq -r '.access_token')

# Wait for token to expire (or use old token)
# For testing, use invalid/expired token
EXPIRED_TOKEN="eyJhbGciOiJIUzI1NiIsInR5cCI6IkpXVCJ9.eyJzdWIiOiIxMjM0NTY3ODkwIiwibmFtZSI6IkpvaG4gRG9lIiwiaWF0IjoxNTE2MjM5MDIyfQ.SflKxwRJSMeKKF2QT4fwpMeJf36POk6yJV_adQssw5c"

curl -s http://localhost:8080/api/v1/admin/users \
  -H "Authorization: Bearer $EXPIRED_TOKEN"
```

**Expected**: `401 Unauthorized`

### 6.3 CORS Validation

#### Test CORS Headers

```bash
curl -v -X OPTIONS http://localhost:8080/api/v1/public/system/info \
  -H "Origin: http://localhost:5173" \
  -H "Access-Control-Request-Method: GET" 2>&1 | grep -i "access-control"
```

**Expected Headers**:

```
Access-Control-Allow-Origin: http://localhost:5173
Access-Control-Allow-Methods: GET, POST, PUT, PATCH, DELETE
Access-Control-Allow-Headers: Content-Type, Authorization
```

### 6.4 SQL Injection Prevention

#### Test SQL Injection in Query Params

```bash
curl -s "http://localhost:8080/api/v1/admin/users?page=1' OR '1'='1" \
  -H "Authorization: Bearer $TOKEN"
```

**Expected**: `400 Bad Request` (parameter validation)

### 6.5 Response Time Benchmarks

#### Health Endpoint

```bash
time curl -s http://localhost:8080/health > /dev/null
```

**Expected**: < 50ms

#### System Info

```bash
time curl -s http://localhost:8080/api/v1/public/system/info > /dev/null
```

**Expected**: < 100ms

#### Authenticated Endpoint

```bash
time curl -s http://localhost:8080/api/v1/admin/users \
  -H "Authorization: Bearer $TOKEN" > /dev/null
```

**Expected**: < 200ms

---

## Validation Checklist

### Phase 0: Initialization (if needed)

- [ ] ✅ Setup status checked
- [ ] ✅ Slug available
- [ ] ✅ System initialized
- [ ] ✅ Admin credentials saved

### Phase 1: System Level

- [ ] ✅ PostgreSQL connected
- [ ] ✅ Redis connected
- [ ] ✅ Backend healthy
- [ ] ✅ System info correct

### Phase 2: Authentication

- [ ] ✅ Login successful
- [ ] ✅ Token valid
- [ ] ✅ Invalid token rejected

### Phase 3: Authorization

- [ ] ✅ Roles listed
- [ ] ✅ Permissions listed
- [ ] ✅ Permission matrix (13/13 tests)

### Phase 4: Business Logic

- [ ] ✅ Users CRUD
- [ ] ✅ Tenants CRUD
- [ ] ✅ Themes management
- [ ] ✅ Media library
- [ ] ✅ CMS pages

### Phase 5: Integration

- [ ] ✅ E2E tests (59/59 pass)
- [ ] ✅ Cross-context validation

---

## Troubleshooting

### Issue: `is_setup: false` แต่ไม่สามารถ initialize ได้

**Solution**:

```bash
# ตรวจสอบ database migrations
docker compose run --rm kyx-kernel sqlx migrate run

# Restart services
docker compose restart
```

### Issue: Cannot login

**Solution**:

```bash
# Check if user exists
psql postgresql://kyx:kyx123@localhost:5432/kyx_db \
  -c "SELECT email, is_active FROM auth_users WHERE email='admin@kyx.tech'"
```

### Issue: 401 on all protected endpoints

**Solution**:

```bash
# Verify token format
echo $TOKEN | cut -d'.' -f2 | base64 -d | jq .

# Check JWT_SECRET in .env
```

---

## Summary

| Phase                     | Tests | Status          |
| ------------------------- | ----- | --------------- |
| 0. Initialization         | 10    | ⚠️ If not setup |
| 1. Health Check           | 3     | ✅ Ready        |
| 2. Authentication         | 3     | ✅ Ready        |
| 3. RBAC                   | 3     | ✅ Ready        |
| 4. Business Logic         | 12+   | ✅ Ready        |
| 5. E2E Integration        | 59    | ✅ Ready        |
| 6. Security & Performance | 8     | ✅ Ready        |

**Total Coverage**: **98+ test cases**

---

## Quick Start

### ถ้าต้องการเริ่มใหม่ทั้งหมด (Fresh Install):

```bash
cd kyx-infra
docker compose down -v
docker compose build
docker compose up -d
sleep 10

# Verify
curl http://localhost:8080/health
curl http://localhost:8080/api/v1/auth/setup/status
```

### ถ้าระบบยังไม่ได้ setup:

```bash
# 1. Check status
curl -s http://localhost:8080/api/v1/auth/setup/status | jq .

# 2. Initialize (ถ้า is_setup: false)
curl -X POST http://localhost:8080/api/v1/auth/setup \
  -H "Content-Type: application/json" \
  -d '{
    "org_name": "KYZ Technologied Co., LTD.",
    "org_slug": "kyz-tech",
    "admin_email": "admin@kyz.tech",
    "admin_password": "SecurePass123!",
    "admin_name": "Admin User"
  }'
```

### ถ้าระบบ setup แล้ว:

```bash
# Run full test suite
./scripts/test-endpoints.sh http://localhost:8080
```

---

**พร้อมทดสอบ! ทำตามลำดับ Phase 0 → Phase 5** 🚀

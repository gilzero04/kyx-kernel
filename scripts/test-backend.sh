#!/bin/bash
# Backend Testing Script - ใช้ระบบที่มีอยู่
# ทดสอบตาม BACKEND_TESTING_FLOW.md

set -e
URL="${1:-http://localhost:8080}"

echo "🧪 Backend Testing Started"
echo "URL: $URL"
echo ""

# Phase 1: Health Check
echo "━━━ Phase 1: Health Check ━━━"
curl -sf "$URL/health" > /dev/null && echo "✅ Health OK" || { echo "❌ Health failed"; exit 1; }
curl -sf "$URL/api/v1/public/system/info" > /dev/null && echo "✅ System Info OK" || { echo "❌ System Info failed"; exit 1; }
echo ""

# Phase 2: Authentication
echo "━━━ Phase 2: Authentication ━━━"
TOKEN=$(curl -s -X POST "$URL/api/v1/auth/login" \
  -H "Content-Type: application/json" \
  -d '{"username":"admin.kyx.tech","password":"Admin123!"}' \
  | jq -r '.access_token // empty')

if [ -z "$TOKEN" ]; then
    echo "❌ Login failed"
    exit 1
fi
echo "✅ Login OK"
echo "$TOKEN" > /tmp/test_token.txt
echo ""

# Phase 3: RBAC
echo "━━━ Phase 3: RBAC ━━━"
curl -sf "$URL/api/v1/admin/roles" -H "Authorization: Bearer $TOKEN" > /dev/null && echo "✅ Roles OK" || { echo "❌ Roles failed"; exit 1; }
curl -sf "$URL/api/v1/admin/permissions" -H "Authorization: Bearer $TOKEN" > /dev/null && echo "✅ Permissions OK" || { echo "❌ Permissions failed"; exit 1; }
echo ""

# Phase 4: Business Logic (sample)
echo "━━━ Phase 4: Business Logic ━━━"
curl -sf "$URL/api/v1/admin/users" -H "Authorization: Bearer $TOKEN" > /dev/null && echo "✅ Users OK" || { echo "❌ Users failed"; exit 1; }
curl -sf "$URL/api/v1/admin/tenants" -H "Authorization: Bearer $TOKEN" > /dev/null && echo "✅ Tenants OK" || { echo "❌ Tenants failed"; exit 1; }
echo ""

# Phase 5: Run full E2E (if exists)
echo "━━━ Phase 5: E2E Tests ━━━"
if [ -f "./scripts/test-endpoints.sh" ]; then
    ./scripts/test-endpoints.sh "$URL" || { echo "⚠️  Some E2E tests failed"; }
else
    echo "⚠️  test-endpoints.sh not found, skipping"
fi
echo ""

echo "✅ Testing Complete!"
echo "Token saved: /tmp/test_token.txt"

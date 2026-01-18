#!/bin/bash
# Simple Backend Testing Script
# ทำตาม BACKEND_TESTING_FLOW.md ทีละ Phase

echo "🔄 เริ่ม Backend Testing..."
echo ""

# Pre-Phase: Fresh Install
echo "━━━ PRE-PHASE: Fresh Installation ━━━"
cd /Users/beykyu/Developer/kyx-tech/kyx-infra
echo "1. Cleaning up..."
docker compose down -v
echo "2. Rebuilding..."
docker compose build
echo "3. Starting..."
docker compose up -d
echo "4. Waiting 15 seconds..."
sleep 15

# Phase 0: Initialize
echo ""
echo "━━━ PHASE 0: Initialize System ━━━"
cd /Users/beykyu/Developer/kyx-tech/kyx-kernel

curl -X POST http://localhost:8080/api/v1/auth/setup \
  -H "Content-Type: application/json" \
  -d '{
    "org_name": "KYZ Technologied Co., LTD.",
    "org_slug": "kyz-tech",
    "admin_email": "admin.kyx.tech",
    "admin_password": "Admin123!",
    "admin_name": "Admin User",
    "platform_type": "multi"
  }' > /tmp/setup_response.json

if grep -q "access_token" /tmp/setup_response.json; then
    echo "✅ Setup successful"
    cat /tmp/setup_response.json | jq -r '.access_token' > /tmp/token.txt
else
    echo "❌ Setup failed"
    cat /tmp/setup_response.json
    exit 1
fi

# Run E2E Tests
echo ""
echo "━━━ Running E2E Tests ━━━"
./scripts/test-endpoints.sh http://localhost:8080

echo ""
echo "✅ Complete!"

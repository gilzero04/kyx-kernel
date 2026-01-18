#!/bin/bash
# Complete Fresh Backend Testing 
# ล้างทุกอย่าง: postgres, redis, และ kernel

set -e

echo "🔄 Starting COMPLETE FRESH Testing..."
echo ""

# ═══════════════════════════════════════════════════════════
# PRE-PHASE: Fresh Installation
# ═══════════════════════════════════════════════════════════
echo "━━━ PRE-PHASE: Complete Fresh Installation ━━━"

# Step 1: Clean kyx-infra services (postgres + redis)
echo "Step 1/8: Cleaning kyx-infra (postgres + redis)..."
cd /Users/beykyu/Developer/kyx-tech/kyx-infra/docker/postgres
docker compose down -v

cd /Users/beykyu/Developer/kyx-tech/kyx-infra/docker/redis
docker compose down -v

# Step 2: Start kyx-infra services
echo "Step 2/8: Starting postgres..."
cd /Users/beykyu/Developer/kyx-tech/kyx-infra/docker/postgres
docker compose up -d

echo "Step 3/8: Starting redis..."
cd /Users/beykyu/Developer/kyx-tech/kyx-infra/docker/redis
docker compose up -d

echo "Step 4/8: Waiting for infra services (10 seconds)..."
sleep 10

# Step 5: Clean kyx-kernel
echo "Step 5/8: Cleaning kyx-kernel..."
cd /Users/beykyu/Developer/kyx-tech/kyx-kernel
docker compose down -v

# Step 6: Rebuild kyx-kernel
echo "Step 6/8: Rebuilding kyx-kernel (no cache)..."
docker compose build --no-cache

# Step 7: Start kyx-kernel
echo "Step 7/8: Starting kyx-kernel..."
docker compose up -d

# Step 8: Wait and verify
echo "Step 8/8: Waiting for backend (20 seconds)..."
sleep 20

echo "Checking health..."
for i in {1..10}; do
    if curl -sf http://localhost:8080/health > /dev/null 2>&1; then
        echo "✅ Backend is healthy"
        break
    fi
    if [ $i -eq 10 ]; then
        echo "❌ Backend failed to start"
        docker compose logs --tail=30
        exit 1
    fi
    sleep 2
done

echo "✅ PRE-PHASE Complete (Fresh postgres, redis, kernel)"
echo ""

# ═══════════════════════════════════════════════════════════
# PHASE 0: System Initialization
# ═══════════════════════════════════════════════════════════
echo "━━━ PHASE 0: System Initialization ━━━"

SETUP_RESPONSE=$(curl -s -X POST http://localhost:8080/api/v1/auth/setup \
  -H "Content-Type: application/json" \
  -d '{
    "org_name": "KYZ Technologied Co., LTD.",
    "org_slug": "kyz-tech",
    "admin_email": "admin.kyx.tech",
    "admin_password": "Admin123!",
    "admin_name": "Admin User",
    "platform_type": "multi"
  }')

TOKEN=$(echo "$SETUP_RESPONSE" | jq -r '.access_token // empty')

if [ -z "$TOKEN" ]; then
    echo "❌ Setup failed"
    echo "$SETUP_RESPONSE" | jq .
    exit 1
fi

echo "✅ System initialized"
echo "$TOKEN" > /tmp/kyx_token.txt
echo ""

# ═══════════════════════════════════════════════════════════
# PHASE 1-5: Run E2E Tests
# ═══════════════════════════════════════════════════════════
echo "━━━ PHASE 1-5: Running E2E Tests ━━━"

if [ -f "./scripts/test-endpoints.sh" ]; then
    ./scripts/test-endpoints.sh http://localhost:8080
else
    echo "⚠️  test-endpoints.sh not found"
fi

echo ""
echo "✅ Testing Complete!"
echo "Token: $(cat /tmp/kyx_token.txt | head -c 50)..."

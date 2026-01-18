#!/bin/bash
# Fresh Deployment - Complete Script
# Pre-Phase (1-8) + Phase 0 (Setup)
set -e

echo "======================================"
echo "🔄 Fresh Deployment + Phase 0"
echo "Start: $(date)"
echo "======================================"

# ═══════════════════════════════════════════════════════════
# PRE-PHASE: Fresh Installation
# ═══════════════════════════════════════════════════════════
echo ""
echo "━━━ PRE-PHASE: Fresh Installation ━━━"

echo ">>> Step 1/8: Cleaning postgres..."
cd /Users/beykyu/Developer/kyx-tech/kyx-infra/docker/postgres
docker compose down -v

echo ">>> Step 2/8: Cleaning redis..."
cd /Users/beykyu/Developer/kyx-tech/kyx-infra/docker/redis
docker compose down -v

echo ">>> Step 3/8: Starting postgres..."
cd /Users/beykyu/Developer/kyx-tech/kyx-infra/docker/postgres
docker compose up -d

echo ">>> Step 4/8: Starting redis..."
cd /Users/beykyu/Developer/kyx-tech/kyx-infra/docker/redis
docker compose up -d

echo ">>> Step 5/9: Waiting 15 seconds for infra..."
sleep 15

echo ">>> Step 6/9: Running database migrations..."
cd /Users/beykyu/Developer/kyx-tech/kyx-kernel
./scripts/migrate-db.sh || {
    echo "❌ Migration failed - stopping before time-consuming build"
    exit 1
}

echo ">>> Step 7/9: Cleaning kernel..."
docker compose down -v

echo ">>> Step 8/9: Building kernel (--no-cache)..."
echo "⚠️  This takes 10-15 minutes..."
export BUILD_DATE=$(date -u +%Y-%m-%dT%H:%M:%SZ)
export GIT_COMMIT=$(git rev-parse --short HEAD 2>/dev/null || echo "dev")
echo "Build Date: $BUILD_DATE"
echo "Git Commit: $GIT_COMMIT"
docker compose build --no-cache

echo ">>> Step 9/9: Starting kernel..."
docker compose up -d
echo "Waiting 20 seconds..."
sleep 20

# Health check
echo ">>> Checking health..."
for i in {1..15}; do
    if curl -sf http://localhost:8080/health; then
        echo ""
        echo "✅ Backend healthy!"
        break
    fi
    [ $i -eq 15 ] && { echo "❌ Health check failed"; docker compose logs --tail=30; exit 1; }
    echo "Attempt $i/15..."
    sleep 3
done

echo ""
echo "✅ PRE-PHASE Complete"

# ═══════════════════════════════════════════════════════════
# PHASE 0: System Initialization
# ═══════════════════════════════════════════════════════════
echo ""
echo "━━━ PHASE 0: System Initialization ━━━"

# Read ENGINE_SECRET_KEY from .env if exists
ENGINE_SECRET=$(grep ENGINE_SECRET_KEY .env 2>/dev/null | cut -d= -f2 || echo "")

echo ">>> Sending setup request..."
if [ -n "$ENGINE_SECRET" ]; then
    echo "Using ENGINE_SECRET_KEY (Production Mode)"
    RESP=$(curl -s -X POST http://localhost:8080/api/v1/auth/setup \
      -H "Content-Type: application/json" \
      -H "X-Engine-Secret: $ENGINE_SECRET" \
      -d '{
        "org_name": "KYZ Technologied Co., LTD.",
        "org_slug": "kyz-tech",
        "email": "admin@kyz.tech",
        "password": "Admin123!",
        "full_name": "Admin User",
        "platform_type": "multi"
      }')
else
    echo "No ENGINE_SECRET_KEY (Development Mode)"
    RESP=$(curl -s -X POST http://localhost:8080/api/v1/auth/setup \
      -H "Content-Type: application/json" \
      -d '{
        "org_name": "KYZ Technologied Co., LTD.",
        "org_slug": "kyz-tech",
        "email": "admin@kyz.tech",
        "password": "Admin123!",
        "full_name": "Admin User",
        "platform_type": "multi"
      }')
fi

echo "Response:"
echo "$RESP" | jq .

TOKEN=$(echo "$RESP" | jq -r '.access_token // empty')

if [ -z "$TOKEN" ]; then
    echo "❌ Setup failed"
    exit 1
fi

echo "$TOKEN" > /tmp/kyx_token.txt
echo ""
echo "✅ Token saved: /tmp/kyx_token.txt"

echo ""
echo "======================================"
echo "✅ DEPLOYMENT COMPLETE!"
echo "End: $(date)"
echo "======================================"
echo ""
echo "Credentials:"
echo "  Email: admin@kyz.tech"
echo "  Password: Admin123!"
echo ""
echo "Next steps:"
echo "  export TOKEN=\$(cat /tmp/kyx_token.txt)"
echo "  curl -H \"Authorization: Bearer \$TOKEN\" http://localhost:8080/api/v1/admin/users"

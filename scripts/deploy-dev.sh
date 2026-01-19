#!/bin/bash
# Development Deployment - Fresh Database + Kernel (No Setup)
# Use this for frontend-driven setup flow
set -e

echo "======================================"
echo "🔄 Development Deployment (No Setup)"
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
echo "======================================"
echo "✅ DEPLOYMENT COMPLETE!"
echo "End: $(date)"
echo "======================================"
echo ""
echo "System Status:"
curl -s http://localhost:8080/api/v1/public/system/status | jq .
echo ""
echo "Next steps:"
echo "  1. Open frontend: http://localhost:5173"
echo "  2. Complete setup wizard"
echo "  3. API available at: http://localhost:8080"

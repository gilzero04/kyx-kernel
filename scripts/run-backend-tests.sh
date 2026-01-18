#!/bin/bash

# ════════════════════════════════════════════════════════════════════════════
# Backend Testing Script - Smart Deployment
# ════════════════════════════════════════════════════════════════════════════
# Purpose: สั่งรัน testing phases แบบอัตโนมัติ
# - ตรวจสอบ setup status ก่อน
# - ถ้า is_setup=false → Fresh install (clear docker)
# - ถ้า is_setup=true → Skip fresh install (ใช้ของเดิม)
# ════════════════════════════════════════════════════════════════════════════

set -e  # Exit on error

BASE_URL="${1:-http://localhost:8080}"
INFRA_DIR="/Users/beykyu/Developer/kyx-tech/kyx-infra"

# Colors
RED='\033[0;31m'
GREEN='\033[0;32m'
YELLOW='\033[1;33m'
BLUE='\033[0;34m'
NC='\033[0m' # No Color

# Helper functions
log_info() {
    echo -e "${BLUE}ℹ️  $1${NC}"
}

log_success() {
    echo -e "${GREEN}✅ $1${NC}"
}

log_warning() {
    echo -e "${YELLOW}⚠️  $1${NC}"
}

log_error() {
    echo -e "${RED}❌ $1${NC}"
}

log_section() {
    echo ""
    echo "━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━"
    echo -e "${BLUE}  $1${NC}"
    echo "━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━"
}

# ════════════════════════════════════════════════════════════════════════════
# STEP 0: Check if Backend is Accessible
# ════════════════════════════════════════════════════════════════════════════

log_section "STEP 0: ตรวจสอบ Backend"

if ! curl -s --max-time 5 "${BASE_URL}/health" > /dev/null 2>&1; then
    log_warning "Backend ไม่ตอบสนอง - จำเป็นต้อง deploy ใหม่"
    NEED_DEPLOY=true
    IS_SETUP=false
else
    log_success "Backend ตอบสนอง"
    
    # Check setup status
    SETUP_RESPONSE=$(curl -s "${BASE_URL}/api/v1/auth/setup/status" 2>/dev/null || echo '{"is_setup":false}')
    IS_SETUP=$(echo "$SETUP_RESPONSE" | jq -r '.is_setup // false')
    
    if [ "$IS_SETUP" = "true" ]; then
        log_success "ระบบ setup แล้ว (is_setup: true)"
        NEED_DEPLOY=false
    else
        log_info "ระบบยังไม่ได้ setup (is_setup: false)"
        NEED_DEPLOY=true
    fi
fi

# ════════════════════════════════════════════════════════════════════════════
# PRE-PHASE: Fresh Installation (ถ้าจำเป็น)
# ════════════════════════════════════════════════════════════════════════════

if [ "$NEED_DEPLOY" = "true" ]; then
    log_section "PRE-PHASE: Fresh Installation"
    
    # Step 1: Clean up
    log_info "Step 1/7: Cleaning up existing infrastructure..."
    cd "$INFRA_DIR"
    docker compose down -v 2>&1 | grep -E "(Removed|Volume)" || true
    log_success "Cleanup complete"
    
    # Step 2: Rebuild
    log_info "Step 2/7: Rebuilding containers..."
    docker compose build --quiet 2>&1 > /dev/null
    log_success "Rebuild complete"
    
    # Step 3: Start services
    log_info "Step 3/7: Starting services..."
    docker compose up -d
    sleep 10
    log_success "Services started"
    
    # Step 4: Verify services
    log_info "Step 4/7: Verifying services..."
    docker compose ps | grep -E "(postgres|redis|kyx-kernel)" || {
        log_error "Services not running properly"
        docker compose logs --tail=50
        exit 1
    }
    log_success "Services verified"
    
    # Step 5: Run migrations (if needed)
    log_info "Step 5/7: Running database migrations..."
    docker compose run --rm kyx-kernel sqlx migrate run 2>&1 | tail -5
    log_success "Migrations complete"
    
    # Step 6: Verify backend health
    log_info "Step 6/7: Verifying backend health..."
    sleep 5
    for i in {1..10}; do
        if curl -s "${BASE_URL}/health" > /dev/null 2>&1; then
            log_success "Backend healthy"
            break
        fi
        if [ $i -eq 10 ]; then
            log_error "Backend health check failed after 10 attempts"
            exit 1
        fi
        sleep 2
    done
    
    # Step 7: Check setup status
    log_info "Step 7/7: Checking setup status..."
    SETUP_STATUS=$(curl -s "${BASE_URL}/api/v1/auth/setup/status" | jq -r '.is_setup')
    if [ "$SETUP_STATUS" = "false" ]; then
        log_success "Setup status: false (พร้อม initialize)"
    else
        log_warning"Setup status: $SETUP_STATUS"
    fi
    
    log_success "PRE-PHASE COMPLETE ✅"
else
    log_section "PRE-PHASE: SKIPPED"
    log_info "ระบบ setup แล้ว - ข้าม Fresh Installation"
fi

# ════════════════════════════════════════════════════════════════════════════
# PHASE 0: System Initialization
# ════════════════════════════════════════════════════════════════════════════

log_section "PHASE 0: System Initialization"

# Check if already setup
CURRENT_SETUP=$(curl -s "${BASE_URL}/api/v1/auth/setup/status" | jq -r '.is_setup')

if [ "$CURRENT_SETUP" = "false" ]; then
    log_info "Initializing system..."
    
    # Development Mode (no engine key)
    INIT_RESPONSE=$(curl -s -X POST "${BASE_URL}/api/v1/auth/setup" \
      -H "Content-Type: application/json" \
      -d '{
        "org_name": "KYZ Technologied Co., LTD.",
        "org_slug": "kyz-tech",
        "admin_email": "admin@kyz.tech",
        "admin_password": "SecurePass123!",
        "admin_name": "Admin User",
        "platform_type": "multi"
      }')
    
    if echo "$INIT_RESPONSE" | jq -e '.access_token' > /dev/null 2>&1; then
        log_success "System initialized successfully"
        TOKEN=$(echo "$INIT_RESPONSE" | jq -r '.access_token')
        echo "$TOKEN" > /tmp/kyx_test_token.txt
        log_info "Token saved to /tmp/kyx_test_token.txt"
    else
        log_error "System initialization failed"
        echo "$INIT_RESPONSE" | jq .
        exit 1
    fi
else
    log_info "System already initialized - logging in..."
    
    # Login to get token
    LOGIN_RESPONSE=$(curl -s -X POST "${BASE_URL}/api/v1/auth/login" \
      -H "Content-Type: application/json" \
      -d '{
        "username": "admin.kyx.tech",
        "password": "SecurePass123!"
      }')
    
    if echo "$LOGIN_RESPONSE" | jq -e '.access_token' > /dev/null 2>&1; then
        log_success "Login successful"
        TOKEN=$(echo "$LOGIN_RESPONSE" | jq -r '.access_token')
        echo "$TOKEN" > /tmp/kyx_test_token.txt
        log_info "Token saved to /tmp/kyx_test_token.txt"
    else
        log_error "Login failed"
        echo "$LOGIN_RESPONSE" | jq .
        exit 1
    fi
fi

log_success "PHASE 0 COMPLETE ✅"

# ════════════════════════════════════════════════════════════════════════════
# SUMMARY
# ════════════════════════════════════════════════════════════════════════════

log_section "TEST SUMMARY"

echo "Base URL: $BASE_URL"
echo "Setup Status: $CURRENT_SETUP"
echo "Token: $(cat /tmp/kyx_test_token.txt | head -c 50)..."
echo ""
echo "Next Steps:"
echo "  export TOKEN=\$(cat /tmp/kyx_test_token.txt)"
echo "  ./scripts/test-endpoints.sh $BASE_URL"
echo ""

log_success "Backend Testing Script Complete! 🎉"

#!/bin/bash
# Backend Step-by-Step Test Script
# ทดสอบทีละขั้นตอน เพิ่มทีละ step ที่ผ่าน

set -e
URL="${1:-http://localhost:8080}"

echo "======================================"
echo "🧪 Backend Step-by-Step Tests"
echo "URL: $URL"
echo "======================================"
echo ""

PASS=0
FAIL=0

# ═══════════════════════════════════════════════════════════
# Step 1: Login Test
# ═══════════════════════════════════════════════════════════
echo "━━━ Step 1: Login Test ━━━"

LOGIN_RESP=$(curl -s -X POST "$URL/api/v1/auth/login" \
  -H "Content-Type: application/json" \
  -d '{"username":"admin@kyz.tech","password":"Admin123!"}')

TOKEN=$(echo "$LOGIN_RESP" | jq -r '.data.access_token // empty')
REFRESH=$(echo "$LOGIN_RESP" | jq -r '.data.refresh_token // empty')
USER_EMAIL=$(echo "$LOGIN_RESP" | jq -r '.data.user.email // empty')
USER_ROLE=$(echo "$LOGIN_RESP" | jq -r '.data.user.role // empty')
PERMISSIONS=$(echo "$LOGIN_RESP" | jq -r '.data.user.permissions | length')
AVATAR=$(echo "$LOGIN_RESP" | jq -r '.data.user.avatar_url // empty')
COVER=$(echo "$LOGIN_RESP" | jq -r '.data.user.cover_url // empty')
IMAGES_COUNT=$(echo "$LOGIN_RESP" | jq -r '.data.user.images | length')
TENANT_TYPE=$(echo "$LOGIN_RESP" | jq -r '.data.user.tenant_type // empty')

# Check access_token
if [ -n "$TOKEN" ]; then
    echo "✅ access_token received"
    PASS=$((PASS + 1))
else
    echo "❌ access_token not received"
    FAIL=$((FAIL + 1))
fi

# Check refresh_token
if [ -n "$REFRESH" ]; then
    echo "✅ refresh_token received"
    PASS=$((PASS + 1))
else
    echo "❌ refresh_token not received"
    FAIL=$((FAIL + 1))
fi

# Check user email
if [ "$USER_EMAIL" = "admin@kyz.tech" ]; then
    echo "✅ user.email = admin@kyz.tech"
    PASS=$((PASS + 1))
else
    echo "❌ user.email = $USER_EMAIL (expected admin@kyz.tech)"
    FAIL=$((FAIL + 1))
fi

# Check user role
if [ "$USER_ROLE" = "superadmin" ]; then
    echo "✅ user.role = superadmin"
    PASS=$((PASS + 1))
else
    echo "❌ user.role = $USER_ROLE (expected superadmin)"
    FAIL=$((FAIL + 1))
fi

# Check permissions count
if [ "$PERMISSIONS" -ge 40 ]; then
    echo "✅ user.permissions = $PERMISSIONS (>= 40)"
    PASS=$((PASS + 1))
else
    echo "❌ user.permissions = $PERMISSIONS (expected >= 40)"
    FAIL=$((FAIL + 1))
fi

# Check auto-generated avatar
if [ -n "$AVATAR" ]; then
    echo "✅ user.avatar_url auto-generated"
    PASS=$((PASS + 1))
else
    echo "❌ user.avatar_url not generated"
    FAIL=$((FAIL + 1))
fi

# Check auto-generated cover
if [ -n "$COVER" ]; then
    echo "✅ user.cover_url auto-generated"
    PASS=$((PASS + 1))
else
    echo "❌ user.cover_url not generated"
    FAIL=$((FAIL + 1))
fi

# Check images array has 2 items
if [ "$IMAGES_COUNT" -ge 2 ]; then
    echo "✅ user.images = $IMAGES_COUNT images (>= 2)"
    PASS=$((PASS + 1))
else
    echo "❌ user.images = $IMAGES_COUNT (expected >= 2)"
    FAIL=$((FAIL + 1))
fi

# Check tenant_type
if [ "$TENANT_TYPE" = "owner" ]; then
    echo "✅ user.tenant_type = owner"
    PASS=$((PASS + 1))
else
    echo "❌ user.tenant_type = $TENANT_TYPE (expected owner)"
    FAIL=$((FAIL + 1))
fi

echo "Token saved to /tmp/kyx_token.txt"
echo "$TOKEN" > /tmp/kyx_token.txt
echo ""

# ═══════════════════════════════════════════════════════════
# Step 2: System Info (Public Endpoint)
# ═══════════════════════════════════════════════════════════
echo "━━━ Step 2: System Info ━━━"

INFO_RESP=$(curl -s "$URL/api/v1/public/system/info")

SYS_NAME=$(echo "$INFO_RESP" | jq -r '.data.name // empty')
SYS_VERSION=$(echo "$INFO_RESP" | jq -r '.data.version // empty')
SYS_ENV=$(echo "$INFO_RESP" | jq -r '.data.environment // empty')
BUILD_COMMIT=$(echo "$INFO_RESP" | jq -r '.data.build.commit // empty')
BUILD_DATE=$(echo "$INFO_RESP" | jq -r '.data.build.date // empty')

# Check system name
if [ "$SYS_NAME" = "Kyx Kernel" ]; then
    echo "✅ name = Kyx Kernel"
    PASS=$((PASS + 1))
else
    echo "❌ name = $SYS_NAME (expected Kyx Kernel)"
    FAIL=$((FAIL + 1))
fi

# Check version exists
if [ -n "$SYS_VERSION" ]; then
    echo "✅ version = $SYS_VERSION"
    PASS=$((PASS + 1))
else
    echo "❌ version not found"
    FAIL=$((FAIL + 1))
fi

# Check environment
if [ -n "$SYS_ENV" ]; then
    echo "✅ environment = $SYS_ENV"
    PASS=$((PASS + 1))
else
    echo "❌ environment not found"
    FAIL=$((FAIL + 1))
fi

# Check build commit (allow 'unknown' in local environment)
if [ -n "$BUILD_COMMIT" ] && [ "$BUILD_COMMIT" != "unknown" ]; then
    echo "✅ build.commit = $BUILD_COMMIT"
    PASS=$((PASS + 1))
elif [ "$SYS_ENV" = "local" ] && [ "$BUILD_COMMIT" = "unknown" ]; then
    echo "✅ build.commit = unknown (expected in local environment)"
    PASS=$((PASS + 1))
else
    echo "❌ build.commit = $BUILD_COMMIT (should not be empty or unknown in production)"
    FAIL=$((FAIL + 1))
fi

# Check build date (allow 'unknown' in local environment)
if [ -n "$BUILD_DATE" ] && [ "$BUILD_DATE" != "unknown" ]; then
    echo "✅ build.date = $BUILD_DATE"
    PASS=$((PASS + 1))
elif [ "$SYS_ENV" = "local" ] && [ "$BUILD_DATE" = "unknown" ]; then
    echo "✅ build.date = unknown (expected in local environment)"
    PASS=$((PASS + 1))
else
    echo "❌ build.date = $BUILD_DATE (should not be empty or unknown in production)"
    FAIL=$((FAIL + 1))
fi
echo ""

# ═══════════════════════════════════════════════════════════
# Step 3: System Status (Public Endpoint)
# ═══════════════════════════════════════════════════════════
echo "━━━ Step 3: System Status ━━━"

STATUS_RESP=$(curl -s "$URL/api/v1/public/system/status")

INSTALLED=$(echo "$STATUS_RESP" | jq -r '.data.installed // empty')
SYS_STATUS=$(echo "$STATUS_RESP" | jq -r '.data.status // empty')

# Check installed = true
if [ "$INSTALLED" = "true" ]; then
    echo "✅ installed = true"
    PASS=$((PASS + 1))
else
    echo "❌ installed = $INSTALLED (expected true)"
    FAIL=$((FAIL + 1))
fi

# Check status = online
if [ "$SYS_STATUS" = "online" ]; then
    echo "✅ status = online"
    PASS=$((PASS + 1))
else
    echo "❌ status = $SYS_STATUS (expected online)"
    FAIL=$((FAIL + 1))
fi
echo ""

# ═══════════════════════════════════════════════════════════
# Step 4: Admin Config (Authenticated)
# ═══════════════════════════════════════════════════════════
echo "━━━ Step 4: Admin Config ━━━"

CONFIG_RESP=$(curl -s "$URL/api/v1/admin/config" -H "Authorization: Bearer $TOKEN")

# New ApiResponse format - config values are in sections
SECTIONS_COUNT=$(echo "$CONFIG_RESP" | jq -r '.data.sections | length // empty')
SUCCESS=$(echo "$CONFIG_RESP" | jq -r '.success // empty')

# Check sections count (should be 5)
if [ "$SECTIONS_COUNT" -ge 5 ]; then
    echo "✅ config.sections count = $SECTIONS_COUNT (>= 5)"
    PASS=$((PASS + 1))
else
    echo "❌ config.sections count = $SECTIONS_COUNT (expected >= 5)"
    FAIL=$((FAIL + 1))
fi

# Check success = true
if [ "$SUCCESS" = "true" ]; then
    echo "✅ config response success = true"
    PASS=$((PASS + 1))
else
    echo "❌ config response success = $SUCCESS (expected true)"
    FAIL=$((FAIL + 1))
fi
echo ""

# ═══════════════════════════════════════════════════════════
# Step 5: Config Update (CRUD - Update)
# ═══════════════════════════════════════════════════════════
echo "━━━ Step 5: Config Update (CRUD) ━━━"

# Update a config value
UPDATE_RESP=$(curl -s -X PATCH "$URL/api/v1/admin/config" \
  -H "Authorization: Bearer $TOKEN" \
  -H "Content-Type: application/json" \
  -d '{"key":"access_token_expire_minutes","value":30}')

UPDATE_SUCCESS=$(echo "$UPDATE_RESP" | jq -r '.success // empty')
UPDATE_KEY=$(echo "$UPDATE_RESP" | jq -r '.data.key // empty')
UPDATE_STATUS=$(echo "$UPDATE_RESP" | jq -r '.data.updated // empty')

# Check update success
if [ "$UPDATE_SUCCESS" = "true" ]; then
    echo "✅ config update success = true"
    PASS=$((PASS + 1))
else
    echo "❌ config update success = $UPDATE_SUCCESS (expected true)"
    FAIL=$((FAIL + 1))
fi

# Check returned key
if [ "$UPDATE_KEY" = "access_token_expire_minutes" ]; then
    echo "✅ config update key = access_token_expire_minutes"
    PASS=$((PASS + 1))
else
    echo "❌ config update key = $UPDATE_KEY (expected access_token_expire_minutes)"
    FAIL=$((FAIL + 1))
fi

# Check updated flag
if [ "$UPDATE_STATUS" = "true" ]; then
    echo "✅ config update updated = true"
    PASS=$((PASS + 1))
else
    echo "❌ config update updated = $UPDATE_STATUS (expected true)"
    FAIL=$((FAIL + 1))
fi
echo ""

# ═══════════════════════════════════════════════════════════
# Step 6: Response Format Validation
# ═══════════════════════════════════════════════════════════
echo "━━━ Step 6: Response Format Validation ━━━"

# Validate login response has standard format
LOGIN_TEST=$(curl -s -X POST "$URL/api/v1/auth/login" \
  -H "Content-Type: application/json" \
  -d '{"username":"admin@kyz.tech","password":"Admin123!"}')

HAS_SUCCESS=$(echo "$LOGIN_TEST" | jq 'has("success")')
HAS_STATUS=$(echo "$LOGIN_TEST" | jq 'has("status")')
HAS_MESSAGE=$(echo "$LOGIN_TEST" | jq 'has("message")')
HAS_DATA=$(echo "$LOGIN_TEST" | jq 'has("data")')
HAS_META=$(echo "$LOGIN_TEST" | jq 'has("meta")')
HAS_TIMESTAMP=$(echo "$LOGIN_TEST" | jq 'has("meta") and (.meta | has("timestamp"))')

# Check success field
if [ "$HAS_SUCCESS" = "true" ]; then
    echo "✅ response has 'success' field"
    PASS=$((PASS + 1))
else
    echo "❌ response missing 'success' field"
    FAIL=$((FAIL + 1))
fi

# Check status field
if [ "$HAS_STATUS" = "true" ]; then
    echo "✅ response has 'status' field"
    PASS=$((PASS + 1))
else
    echo "❌ response missing 'status' field"
    FAIL=$((FAIL + 1))
fi

# Check message field
if [ "$HAS_MESSAGE" = "true" ]; then
    echo "✅ response has 'message' field"
    PASS=$((PASS + 1))
else
    echo "❌ response missing 'message' field"
    FAIL=$((FAIL + 1))
fi

# Check data field
if [ "$HAS_DATA" = "true" ]; then
    echo "✅ response has 'data' field"
    PASS=$((PASS + 1))
else
    echo "❌ response missing 'data' field"
    FAIL=$((FAIL + 1))
fi

# Check meta.timestamp
if [ "$HAS_TIMESTAMP" = "true" ]; then
    echo "✅ response has 'meta.timestamp'"
    PASS=$((PASS + 1))
else
    echo "❌ response missing 'meta.timestamp'"
    FAIL=$((FAIL + 1))
fi
echo ""

# ═══════════════════════════════════════════════════════════
# Step 7: Roles Full CRUD (List, Create, Update, Delete)
# ═══════════════════════════════════════════════════════════
echo "━━━ Step 7: Roles CRUD ━━━"

# 7.1 List Roles
ROLES_RESP=$(curl -s -X GET "$URL/api/v1/admin/roles" \
  -H "Authorization: Bearer $TOKEN")

ROLES_SUCCESS=$(echo "$ROLES_RESP" | jq -r '.success // empty')
ROLES_COUNT=$(echo "$ROLES_RESP" | jq -r '.data.roles | length // 0')
ROLES_HAS_META=$(echo "$ROLES_RESP" | jq 'has("meta")')

if [ "$ROLES_SUCCESS" = "true" ]; then
    echo "✅ GET /admin/roles success = true"
    PASS=$((PASS + 1))
else
    echo "❌ GET /admin/roles success = $ROLES_SUCCESS"
    FAIL=$((FAIL + 1))
fi

if [ "$ROLES_COUNT" -ge 1 ]; then
    echo "✅ roles count = $ROLES_COUNT (>= 1)"
    PASS=$((PASS + 1))
else
    echo "❌ roles count = $ROLES_COUNT (expected >= 1)"
    FAIL=$((FAIL + 1))
fi

if [ "$ROLES_HAS_META" = "true" ]; then
    echo "✅ roles has 'meta' field (ApiResponse format)"
    PASS=$((PASS + 1))
else
    echo "❌ roles missing 'meta' field"
    FAIL=$((FAIL + 1))
fi

# 7.2 Create Role (use timestamp for unique slug)
TEST_ROLE_SLUG="test-role-$(date +%s)"
ROLE_CREATE=$(curl -s -X POST "$URL/api/v1/admin/roles" \
    -H "Authorization: Bearer $TOKEN" \
    -H "Content-Type: application/json" \
    -d "{\"name\": \"Test Role CRUD\", \"slug\": \"$TEST_ROLE_SLUG\", \"description\": \"Created by test script\"}")

ROLE_CREATE_SUCCESS=$(echo "$ROLE_CREATE" | jq -r '.success // empty')
ROLE_NEW_ID=$(echo "$ROLE_CREATE" | jq -r '.data.id // empty')

if [ "$ROLE_CREATE_SUCCESS" = "true" ] && [ -n "$ROLE_NEW_ID" ]; then
    echo "✅ POST /admin/roles created ID = ${ROLE_NEW_ID:0:8}..."
    PASS=$((PASS + 1))
    
    # 7.3 Update Role (Note: router uses PATCH not PUT!)
    ROLE_UPDATE=$(curl -s -X PATCH "$URL/api/v1/admin/roles/$ROLE_NEW_ID" \
        -H "Authorization: Bearer $TOKEN" \
        -H "Content-Type: application/json" \
        -d '{"name": "Test Role Updated", "description": "Updated by test script"}')
    
    ROLE_UPDATE_SUCCESS=$(echo "$ROLE_UPDATE" | jq -r '.success // empty')
    
    if [ "$ROLE_UPDATE_SUCCESS" = "true" ]; then
        echo "✅ PATCH /admin/roles/{id} success = true"
        PASS=$((PASS + 1))
    else
        echo "❌ PATCH /admin/roles/{id} success = $ROLE_UPDATE_SUCCESS"
        FAIL=$((FAIL + 1))
    fi
    
    # 7.4 Delete Role
    ROLE_DELETE=$(curl -s -X DELETE "$URL/api/v1/admin/roles/$ROLE_NEW_ID" \
        -H "Authorization: Bearer $TOKEN")
    
    ROLE_DELETE_SUCCESS=$(echo "$ROLE_DELETE" | jq -r '.success // empty')
    
    if [ "$ROLE_DELETE_SUCCESS" = "true" ]; then
        echo "✅ DELETE /admin/roles/{id} success = true"
        PASS=$((PASS + 1))
    else
        echo "❌ DELETE /admin/roles/{id} success = $ROLE_DELETE_SUCCESS"
        FAIL=$((FAIL + 1))
    fi
    
else
    echo "❌ POST /admin/roles failed to create (skip update/delete)"
    FAIL=$((FAIL + 3))
fi
echo ""

# ═══════════════════════════════════════════════════════════
# Step 8: Permissions List
# ═══════════════════════════════════════════════════════════
echo "━━━ Step 8: Permissions List ━━━"

PERMS_RESP=$(curl -s -X GET "$URL/api/v1/admin/permissions" \
  -H "Authorization: Bearer $TOKEN")

PERMS_SUCCESS=$(echo "$PERMS_RESP" | jq -r '.success // empty')
PERMS_COUNT=$(echo "$PERMS_RESP" | jq -r '.data.permissions | length // 0')

if [ "$PERMS_SUCCESS" = "true" ]; then
    echo "✅ permissions response success = true"
    PASS=$((PASS + 1))
else
    echo "❌ permissions response success = $PERMS_SUCCESS (expected true)"
    FAIL=$((FAIL + 1))
fi

if [ "$PERMS_COUNT" -ge 1 ]; then
    echo "✅ permissions count = $PERMS_COUNT (>= 1)"
    PASS=$((PASS + 1))
else
    echo "❌ permissions count = $PERMS_COUNT (expected >= 1)"
    FAIL=$((FAIL + 1))
fi
echo ""

# ═══════════════════════════════════════════════════════════
# Step 9: Tenants List
# ═══════════════════════════════════════════════════════════
echo "━━━ Step 9: Tenants List ━━━"

TENANTS_RESP=$(curl -s -X GET "$URL/api/v1/admin/tenants" \
  -H "Authorization: Bearer $TOKEN")

TENANTS_SUCCESS=$(echo "$TENANTS_RESP" | jq -r '.success // empty')
TENANTS_COUNT=$(echo "$TENANTS_RESP" | jq -r '.data.tenants | length // 0')

if [ "$TENANTS_SUCCESS" = "true" ]; then
    echo "✅ tenants response success = true"
    PASS=$((PASS + 1))
else
    echo "❌ tenants response success = $TENANTS_SUCCESS (expected true)"
    FAIL=$((FAIL + 1))
fi

if [ "$TENANTS_COUNT" -ge 1 ]; then
    echo "✅ tenants count = $TENANTS_COUNT (>= 1)"
    PASS=$((PASS + 1))
else
    echo "❌ tenants count = $TENANTS_COUNT (expected >= 1)"
    FAIL=$((FAIL + 1))
fi
echo ""

# ═══════════════════════════════════════════════════════════
# Step 10: CORS Full CRUD (List, Create, Update, Delete)
# ═══════════════════════════════════════════════════════════
echo "━━━ Step 10: CORS CRUD ━━━"

# 10.1 List CORS
CORS_RESP=$(curl -s -X GET "$URL/api/v1/admin/cors" \
  -H "Authorization: Bearer $TOKEN")

CORS_SUCCESS=$(echo "$CORS_RESP" | jq -r '.success // empty')
CORS_COUNT=$(echo "$CORS_RESP" | jq -r '.data.origins | length // 0')

if [ "$CORS_SUCCESS" = "true" ]; then
    echo "✅ GET /admin/cors success = true"
    PASS=$((PASS + 1))
else
    echo "❌ GET /admin/cors success = $CORS_SUCCESS"
    FAIL=$((FAIL + 1))
fi

if [ "$CORS_COUNT" -ge 1 ]; then
    echo "✅ cors origins count = $CORS_COUNT (>= 1)"
    PASS=$((PASS + 1))
else
    echo "❌ cors origins count = $CORS_COUNT (expected >= 1)"
    FAIL=$((FAIL + 1))
fi

# 10.2 Create CORS Origin (use timestamp for unique origin)
TEST_ORIGIN="https://test-crud-$(date +%s).example.com"
CORS_CREATE=$(curl -s -X POST "$URL/api/v1/admin/cors" \
    -H "Authorization: Bearer $TOKEN" \
    -H "Content-Type: application/json" \
    -d "{\"origin\": \"$TEST_ORIGIN\", \"description\": \"Test CRUD origin\"}")

CORS_CREATE_SUCCESS=$(echo "$CORS_CREATE" | jq -r '.success // empty')
CORS_NEW_ID=$(echo "$CORS_CREATE" | jq -r '.data.id // empty')

if [ "$CORS_CREATE_SUCCESS" = "true" ] && [ -n "$CORS_NEW_ID" ]; then
    echo "✅ POST /admin/cors created ID = ${CORS_NEW_ID:0:8}..."
    PASS=$((PASS + 1))
    
    # 10.3 Update CORS Origin (Note: router uses PATCH not PUT!)
    CORS_UPDATE=$(curl -s -X PATCH "$URL/api/v1/admin/cors/$CORS_NEW_ID" \
        -H "Authorization: Bearer $TOKEN" \
        -H "Content-Type: application/json" \
        -d '{"is_active": true, "description": "Updated test CRUD origin"}')
    
    CORS_UPDATE_SUCCESS=$(echo "$CORS_UPDATE" | jq -r '.success // empty')
    
    if [ "$CORS_UPDATE_SUCCESS" = "true" ]; then
        echo "✅ PATCH /admin/cors/{id} success = true"
        PASS=$((PASS + 1))
    else
        echo "❌ PATCH /admin/cors/{id} success = $CORS_UPDATE_SUCCESS"
        FAIL=$((FAIL + 1))
    fi
    
    # 10.4 Delete CORS Origin (use i32 not UUID!)
    CORS_DELETE=$(curl -s -X DELETE "$URL/api/v1/admin/cors/$CORS_NEW_ID" \
        -H "Authorization: Bearer $TOKEN")
    
    CORS_DELETE_SUCCESS=$(echo "$CORS_DELETE" | jq -r '.success // empty')
    
    if [ "$CORS_DELETE_SUCCESS" = "true" ]; then
        echo "✅ DELETE /admin/cors/{id} success = true"
        PASS=$((PASS + 1))
    else
        echo "❌ DELETE /admin/cors/{id} success = $CORS_DELETE_SUCCESS"
        FAIL=$((FAIL + 1))
    fi
    
else
    echo "❌ POST /admin/cors failed to create (skip update/delete)"
    FAIL=$((FAIL + 3))
fi
echo ""

# ═══════════════════════════════════════════════════════════
# Step 11: i18n Locales (Public)
# ═══════════════════════════════════════════════════════════
echo "━━━ Step 11: i18n Locales ━━━"

I18N_RESP=$(curl -s -X GET "$URL/api/v1/public/i18n/locales")

I18N_SUCCESS=$(echo "$I18N_RESP" | jq -r '.success // empty')
I18N_COUNT=$(echo "$I18N_RESP" | jq -r '.data.locales | length // 0')

if [ "$I18N_SUCCESS" = "true" ]; then
    echo "✅ i18n locales response success = true"
    PASS=$((PASS + 1))
else
    echo "❌ i18n locales response success = $I18N_SUCCESS (expected true)"
    FAIL=$((FAIL + 1))
fi

if [ "$I18N_COUNT" -ge 1 ]; then
    echo "✅ i18n locales count = $I18N_COUNT (>= 1)"
    PASS=$((PASS + 1))
else
    echo "❌ i18n locales count = $I18N_COUNT (expected >= 1)"
    FAIL=$((FAIL + 1))
fi
echo ""

# ═══════════════════════════════════════════════════════════
# Step 12: Unified Branding (Public)
# ═══════════════════════════════════════════════════════════
echo "━━━ Step 12: Unified Branding ━━━"

BRANDING_RESP=$(curl -s -X GET "$URL/api/v1/public/branding")

BRANDING_SUCCESS=$(echo "$BRANDING_RESP" | jq -r '.success // empty')
BRANDING_SLUG=$(echo "$BRANDING_RESP" | jq -r '.data.tenant.slug // empty')
BRANDING_CONTEXT=$(echo "$BRANDING_RESP" | jq -r '.data.context // empty')

if [ "$BRANDING_SUCCESS" = "true" ]; then
    echo "✅ branding response success = true"
    PASS=$((PASS + 1))
else
    echo "❌ branding response success = $BRANDING_SUCCESS (expected true)"
    FAIL=$((FAIL + 1))
fi

if [ -n "$BRANDING_SLUG" ]; then
    echo "✅ branding has tenant.slug = $BRANDING_SLUG"
    PASS=$((PASS + 1))
else
    echo "❌ branding missing tenant.slug"
    FAIL=$((FAIL + 1))
fi

if [ "$BRANDING_CONTEXT" = "console" ]; then
    echo "✅ branding context = console (default)"
    PASS=$((PASS + 1))
else
    echo "❌ branding context = $BRANDING_CONTEXT (expected console)"
    FAIL=$((FAIL + 1))
fi
echo ""

# ═══════════════════════════════════════════════════════════
# Step 13: Themes CRUD (Public + Admin)
# ═══════════════════════════════════════════════════════════
echo "━━━ Step 13: Themes CRUD ━━━"

# 13.1 List Themes (Public)
THEMES_RESP=$(curl -s -X GET "$URL/api/v1/public/themes")
THEMES_SUCCESS=$(echo "$THEMES_RESP" | jq -r '.success // empty')
THEMES_COUNT=$(echo "$THEMES_RESP" | jq -r '.data.themes | length // 0')

if [ "$THEMES_SUCCESS" = "true" ]; then
    echo "✅ GET /public/themes success = true"
    PASS=$((PASS + 1))
else
    echo "❌ GET /public/themes success = $THEMES_SUCCESS (expected true)"
    FAIL=$((FAIL + 1))
fi

if [ "$THEMES_COUNT" -ge 1 ]; then
    echo "✅ themes count = $THEMES_COUNT (>= 1)"
    PASS=$((PASS + 1))
else
    echo "❌ themes count = $THEMES_COUNT (expected >= 1)"
    FAIL=$((FAIL + 1))
fi

# 13.2 Get first theme ID for activate test
THEME_ID=$(echo "$THEMES_RESP" | jq -r '.data.themes[0].id // empty')

if [ -n "$THEME_ID" ]; then
    echo "✅ themes has valid UUID = ${THEME_ID:0:8}..."
    PASS=$((PASS + 1))
    
    # 13.3 Activate Theme (PATCH)
    ACTIVATE_RESP=$(curl -s -X PATCH "$URL/api/v1/admin/themes/$THEME_ID/activate" \
        -H "Authorization: Bearer $TOKEN" \
        -H "Content-Type: application/json" \
        -d '{"mode": "light"}')
    
    ACTIVATE_SUCCESS=$(echo "$ACTIVATE_RESP" | jq -r '.success // empty')
    ACTIVATE_MODE=$(echo "$ACTIVATE_RESP" | jq -r '.data.mode // empty')
    
    if [ "$ACTIVATE_SUCCESS" = "true" ]; then
        echo "✅ PATCH /admin/themes/{id}/activate success = true"
        PASS=$((PASS + 1))
    else
        echo "❌ PATCH /admin/themes/{id}/activate success = $ACTIVATE_SUCCESS"
        FAIL=$((FAIL + 1))
    fi
    
    if [ "$ACTIVATE_MODE" = "light" ]; then
        echo "✅ theme activated mode = light"
        PASS=$((PASS + 1))
    else
        echo "❌ theme activated mode = $ACTIVATE_MODE (expected light)"
        FAIL=$((FAIL + 1))
    fi

else
    echo "❌ themes missing valid ID for CRUD tests"
    FAIL=$((FAIL + 3))
fi
echo ""

# ═══════════════════════════════════════════════════════════
# Step 14: i18n Translations (Public)
# ═══════════════════════════════════════════════════════════
echo "━━━ Step 14: i18n Translations ━━━"

TRANS_RESP=$(curl -s -X GET "$URL/api/v1/public/i18n/en")
TRANS_SUCCESS=$(echo "$TRANS_RESP" | jq -r '.success // empty')
TRANS_LOCALE=$(echo "$TRANS_RESP" | jq -r '.data.locale // empty')

if [ "$TRANS_SUCCESS" = "true" ]; then
    echo "✅ GET /public/i18n/en success = true"
    PASS=$((PASS + 1))
else
    echo "❌ GET /public/i18n/en success = $TRANS_SUCCESS"
    FAIL=$((FAIL + 1))
fi

if [ "$TRANS_LOCALE" = "en" ]; then
    echo "✅ translations locale = en"
    PASS=$((PASS + 1))
else
    echo "❌ translations locale = $TRANS_LOCALE (expected en)"
    FAIL=$((FAIL + 1))
fi
echo ""

# ═══════════════════════════════════════════════════════════
# Step 15: Tenants/Me (Admin)
# ═══════════════════════════════════════════════════════════
echo "━━━ Step 15: Tenants/Me ━━━"

ME_RESP=$(curl -s -X GET "$URL/api/v1/admin/tenants/me" \
    -H "Authorization: Bearer $TOKEN")
ME_SUCCESS=$(echo "$ME_RESP" | jq -r '.success // empty')
ME_TENANT_ID=$(echo "$ME_RESP" | jq -r '.data.id // empty')

if [ "$ME_SUCCESS" = "true" ]; then
    echo "✅ GET /admin/tenants/me success = true"
    PASS=$((PASS + 1))
else
    echo "❌ GET /admin/tenants/me success = $ME_SUCCESS"
    FAIL=$((FAIL + 1))
fi

if [ -n "$ME_TENANT_ID" ]; then
    echo "✅ tenants/me returns tenant.id = ${ME_TENANT_ID:0:8}..."
    PASS=$((PASS + 1))
else
    echo "❌ tenants/me missing tenant.id"
    FAIL=$((FAIL + 1))
fi
echo ""

# ═══════════════════════════════════════════════════════════
# Step 16: Theme Sharing (Admin)
# ═══════════════════════════════════════════════════════════
echo "━━━ Step 16: Theme Sharing ━━━"

# Try to update sharing (may fail if system theme, but endpoint should respond)
SHARE_RESP=$(curl -s -X PATCH "$URL/api/v1/admin/themes/$THEME_ID/sharing" \
    -H "Authorization: Bearer $TOKEN" \
    -H "Content-Type: application/json" \
    -d '{"is_shared": true}')

SHARE_SUCCESS=$(echo "$SHARE_RESP" | jq -r '.success // empty')
SHARE_STATUS=$(echo "$SHARE_RESP" | jq -r '.status // 0')

# System themes return 403, custom themes return 200
if [ "$SHARE_SUCCESS" = "true" ] || [ "$SHARE_STATUS" = "403" ]; then
    echo "✅ PATCH /admin/themes/{id}/sharing endpoint responds (success=$SHARE_SUCCESS status=$SHARE_STATUS)"
    PASS=$((PASS + 1))
else
    echo "❌ PATCH /admin/themes/{id}/sharing unexpected response"
    FAIL=$((FAIL + 1))
fi
echo ""

# ═══════════════════════════════════════════════════════════
# Step 17: Auth Refresh Token
# ═══════════════════════════════════════════════════════════
echo "━━━ Step 17: Auth Refresh ━━━"

REFRESH_RESP=$(curl -s -X POST "$URL/api/v1/auth/refresh" \
    -H "Content-Type: application/json" \
    -d "{\"refresh_token\": \"$REFRESH\"}")

REFRESH_SUCCESS=$(echo "$REFRESH_RESP" | jq -r '.success // empty')
NEW_ACCESS=$(echo "$REFRESH_RESP" | jq -r '.data.access_token // empty')

if [ "$REFRESH_SUCCESS" = "true" ]; then
    echo "✅ POST /auth/refresh success = true"
    PASS=$((PASS + 1))
else
    echo "❌ POST /auth/refresh success = $REFRESH_SUCCESS"
    FAIL=$((FAIL + 1))
fi

if [ -n "$NEW_ACCESS" ]; then
    echo "✅ new access_token received"
    PASS=$((PASS + 1))
else
    echo "❌ new access_token not received"
    FAIL=$((FAIL + 1))
fi
echo ""

# ═══════════════════════════════════════════════════════════
# Step 19: Logout
# ═══════════════════════════════════════════════════════════
echo "━━━ Step 19: Logout ━━━"

# Use the current access token from login
LOGOUT_RESP=$(curl -s -X POST "$URL/api/v1/auth/logout" \
    -H "Authorization: Bearer $TOKEN")

LOGOUT_SUCCESS=$(echo "$LOGOUT_RESP" | jq -r '.success // empty')
LOGOUT_DATA=$(echo "$LOGOUT_RESP" | jq -r '.data.logged_out // empty')

if [ "$LOGOUT_SUCCESS" = "true" ]; then
    echo "✅ POST /auth/logout success = true"
    PASS=$((PASS + 1))
else
    echo "❌ POST /auth/logout success = $LOGOUT_SUCCESS"
    FAIL=$((FAIL + 1))
fi

if [ "$LOGOUT_DATA" = "true" ]; then
    echo "✅ logout data.logged_out = true"
    PASS=$((PASS + 1))
else
    echo "❌ logout data.logged_out = $LOGOUT_DATA"
    FAIL=$((FAIL + 1))
fi

# Verify token is now invalid by trying to use it
INVALID_RESP=$(curl -s -w "\nHTTP_CODE:%{http_code}" -X GET "$URL/api/v1/admin/tenants/me" \
    -H "Authorization: Bearer $TOKEN")
INVALID_CODE=$(echo "$INVALID_RESP" | grep "HTTP_CODE" | cut -d':' -f2)

if [ "$INVALID_CODE" = "401" ] || [ "$INVALID_CODE" = "403" ]; then
    echo "✅ token invalidated after logout ($INVALID_CODE)"
    PASS=$((PASS + 1))
else
    echo "❌ token still valid after logout (HTTP=$INVALID_CODE)"
    FAIL=$((FAIL + 1))
fi

# Re-login for subsequent tests
echo "Re-logging in for subsequent tests..."
LOGIN_AFTER=$(curl -s -X POST "$URL/api/v1/auth/login" \
    -H "Content-Type: application/json" \
    -d '{"username":"admin@kyz.tech","password":"Admin123!"}')
TOKEN=$(echo "$LOGIN_AFTER" | jq -r '.data.access_token // empty')
echo ""

# ═══════════════════════════════════════════════════════════
# Step 20: User Session Management
# ═══════════════════════════════════════════════════════════
echo "━━━ Step 20: User Session Management ━━━"

# 20.1 List User Sessions
SESSIONS_RESP=$(curl -s -X GET "$URL/api/v1/auth/sessions" \
    -H "Authorization: Bearer $TOKEN")

SESSIONS_SUCCESS=$(echo "$SESSIONS_RESP" | jq -r '.success // empty')
SESSIONS_DATA=$(echo "$SESSIONS_RESP" | jq -r '.data.sessions // empty')
SESSIONS_COUNT=$(echo "$SESSIONS_RESP" | jq -r '.data.sessions | length // 0')

if [ "$SESSIONS_SUCCESS" = "true" ]; then
    echo "✅ GET /auth/sessions success = true"
    PASS=$((PASS + 1))
else
    echo "❌ GET /auth/sessions success = $SESSIONS_SUCCESS"
    FAIL=$((FAIL + 1))
fi

if [ "$SESSIONS_COUNT" -ge 1 ]; then
    echo "✅ sessions count = $SESSIONS_COUNT (>= 1)"
    PASS=$((PASS + 1))
else
    echo "❌ sessions count = $SESSIONS_COUNT (expected >= 1)"
    FAIL=$((FAIL + 1))
fi

# 20.2 Extract first session ID for revoke test
FIRST_SID=$(echo "$SESSIONS_RESP" | jq -r '.data.sessions[0].sid // empty')

if [ -n "$FIRST_SID" ]; then
    # Try to revoke a non-current session (or create a new session first)
    # For this test, we'll create a second session
    LOGIN_SECOND=$(curl -s -X POST "$URL/api/v1/auth/login" \
        -H "Content-Type: application/json" \
        -d '{"username":"admin@kyz.tech","password":"Admin123!"}')
    SECOND_TOKEN=$(echo "$LOGIN_SECOND" | jq -r '.data.access_token // empty')
    
    # Get the session ID from second token's claims (extract from token)
    # For simplicity, revoke using the SECOND_TOKEN to revoke its own session
    REVOKE_RESP=$(curl -s -X DELETE "$URL/api/v1/auth/sessions/$FIRST_SID" \
        -H "Authorization: Bearer $TOKEN")
    
    REVOKE_SUCCESS=$(echo "$REVOKE_RESP" | jq -r '.success // empty')
    REVOKE_DATA=$(echo "$REVOKE_RESP" | jq -r '.data.revoked // empty')
    
    if [ "$REVOKE_SUCCESS" = "true" ]; then
        echo "✅ DELETE /auth/sessions/{sid} success = true"
        PASS=$((PASS + 1))
    else
        echo "❌ DELETE /auth/sessions/{sid} success = $REVOKE_SUCCESS"
        FAIL=$((FAIL + 1))
    fi
    
    if [ "$REVOKE_DATA" = "true" ]; then
        echo "✅ session revoked (data.revoked = true)"
        PASS=$((PASS + 1))
    else
        echo "❌ session revoke data = $REVOKE_DATA"
        FAIL=$((FAIL + 1))
    fi
else
    echo "❌ No sessions found to test revoke (skipping revoke tests)"
    FAIL=$((FAIL + 2))
fi
echo ""

# ═══════════════════════════════════════════════════════════
# Step 21: Profile Management
# ═══════════════════════════════════════════════════════════
echo "━━━ Step 21: Profile Management ━━━"

# 21.1 Get Current User Profile
ME_RESP=$(curl -s -X GET "$URL/api/v1/auth/me" \
    -H "Authorization: Bearer $TOKEN")

ME_SUCCESS=$(echo "$ME_RESP" | jq -r '.success // empty')
ME_EMAIL=$(echo "$ME_RESP" | jq -r '.data.email // empty')
ME_USER_ID=$(echo "$ME_RESP" | jq -r '.data.id // empty')

if [ "$ME_SUCCESS" = "true" ]; then
    echo "✅ GET /auth/me success = true"
    PASS=$((PASS + 1))
else
    echo "❌ GET /auth/me success = $ME_SUCCESS"
    FAIL=$((FAIL + 1))
fi

if [ -n "$ME_EMAIL" ]; then
    echo "✅ profile has email = $ME_EMAIL"
    PASS=$((PASS + 1))
else
    echo "❌ profile email missing"
    FAIL=$((FAIL + 1))
fi

# 21.2 Update Profile (full_name)
PROFILE_UPDATE_RESP=$(curl -s -X PUT "$URL/api/v1/auth/me/profile" \
    -H "Authorization: Bearer $TOKEN" \
    -H "Content-Type: application/json" \
    -d '{"full_name":"Test User Updated","cover_url":"https://example.com/cover.jpg"}')

PROFILE_UPDATE_SUCCESS=$(echo "$PROFILE_UPDATE_RESP" | jq -r '.success // empty')
PROFILE_UPDATED=$(echo "$PROFILE_UPDATE_RESP" | jq -r '.data.updated // empty')

if [ "$PROFILE_UPDATE_SUCCESS" = "true" ]; then
    echo "✅ PUT /auth/me/profile success = true"
    PASS=$((PASS + 1))
else
    echo "❌ PUT /auth/me/profile success = $PROFILE_UPDATE_SUCCESS"
    FAIL=$((FAIL + 1))
fi

if [ "$PROFILE_UPDATED" = "true" ]; then
    echo "✅ profile updated (data.updated = true)"
    PASS=$((PASS + 1))
else
    echo "❌ profile update data = $PROFILE_UPDATED"
    FAIL=$((FAIL + 1))
fi

# 21.3 Update Avatar
AVATAR_UPDATE_RESP=$(curl -s -X PUT "$URL/api/v1/auth/me/avatar" \
    -H "Authorization: Bearer $TOKEN" \
    -H "Content-Type: application/json" \
    -d '{"avatar_url":"https://example.com/avatar.jpg"}')

AVATAR_SUCCESS=$(echo "$AVATAR_UPDATE_RESP" | jq -r '.success // empty')
AVATAR_URL=$(echo "$AVATAR_UPDATE_RESP" | jq -r '.data.avatar_url // empty')

if [ "$AVATAR_SUCCESS" = "true" ]; then
    echo "✅ PUT /auth/me/avatar success = true"
    PASS=$((PASS + 1))
else
    echo "❌ PUT /auth/me/avatar success = $AVATAR_SUCCESS"
    FAIL=$((FAIL + 1))
fi

if [ -n "$AVATAR_URL" ]; then
    echo "✅ avatar updated (avatar_url returned)"
    PASS=$((PASS + 1))
else
    echo "❌ avatar_url missing"
    FAIL=$((FAIL + 1))
fi
echo ""

# ═══════════════════════════════════════════════════════════
# Step 22: Permissions CRUD (Phase 1 - not yet implemented handlers)
# Note: Skipping for now - handlers need ApiResponse wrapper first
# ═══════════════════════════════════════════════════════════

# ═══════════════════════════════════════════════════════════
# Step 23: Admin Sessions (Phase 1 - not yet implemented)
# Note: Skipping for now - handlers need ApiResponse wrapper first
# ═══════════════════════════════════════════════════════════

# ═══════════════════════════════════════════════════════════
# Step 24: Context Endpoint
# ═══════════════════════════════════════════════════════════
echo "━━━ Step 22: Context Endpoint ━━━"

CONTEXT_RESP=$(curl -s -X GET "$URL/api/v1/admin/context" \
    -H "Authorization: Bearer $TOKEN")

CONTEXT_SUCCESS=$(echo "$CONTEXT_RESP" | jq -r '.success // empty')
CONTEXT_OWNER=$(echo "$CONTEXT_RESP" | jq -r '.data.owner_id // empty')

if [ "$CONTEXT_SUCCESS" = "true" ]; then
    echo "✅ GET /admin/context success = true"
    PASS=$((PASS + 1))
else
    echo "❌ GET /admin/context success = $CONTEXT_SUCCESS"
    FAIL=$((FAIL + 1))
fi

if [ -n "$CONTEXT_OWNER" ]; then
    echo "✅ context has owner_id for hierarchy"
    PASS=$((PASS + 1))
else
    echo "❌ owner_id missing from context"
    FAIL=$((FAIL + 1))
fi
echo ""

# ═══════════════════════════════════════════════════════════
# Step 18: Theme Import CRUD (Import, Update, Delete)
# ═══════════════════════════════════════════════════════════
echo "━━━ Step 18: Theme Import CRUD ━━━"
SCRIPT_DIR="$(dirname "$0")"

# 18.1 Generate unique manifest.json with timestamp
TEST_THEME_UUID=$(uuidgen 2>/dev/null || cat /proc/sys/kernel/random/uuid 2>/dev/null || echo "$(date +%s)-test-uuid")
TEST_THEME_UUID=$(echo "$TEST_THEME_UUID" | tr '[:upper:]' '[:lower:]')
TEST_THEME_SLUG="test-theme-$(date +%s)"

# Check if fixtures folder exists
if [ ! -d "$SCRIPT_DIR/fixtures/test-theme-crud" ]; then
    echo "❌ Missing scripts/fixtures/test-theme-crud folder"
    FAIL=$((FAIL + 3))
else
    # Create manifest.json with unique slug AND name
    cat > "$SCRIPT_DIR/fixtures/test-theme-crud/manifest.json" << EOF
{
    "id": "$TEST_THEME_UUID",
    "slug": "$TEST_THEME_SLUG",
    "name": "Test Theme $TEST_THEME_SLUG",
    "type": "light",
    "description": "Test theme for CRUD operations",
    "version": "1.0.0",
    "author": "Test Script",
    "is_system": false,
    "preview": "images/preview.png",
    "logo": "images/logo.png",
    "config": {}
}
EOF
    
    # Create ZIP with files at root level
    cd "$SCRIPT_DIR/fixtures/test-theme-crud" && zip -rq ../test-theme-crud.zip . && cd - > /dev/null
    
    # 18.2 Import Theme
    IMPORT_RESP=$(curl -s -X POST "$URL/api/v1/admin/themes/import" \
        -H "Authorization: Bearer $TOKEN" \
        -F "file=@$SCRIPT_DIR/fixtures/test-theme-crud.zip")
    
    IMPORT_SUCCESS=$(echo "$IMPORT_RESP" | jq -r '.success // empty')
    IMPORTED_ID=$(echo "$IMPORT_RESP" | jq -r '.data.id // empty')
    
    if [ "$IMPORT_SUCCESS" = "true" ] && [ -n "$IMPORTED_ID" ]; then
        echo "✅ POST /admin/themes/import success, ID = ${IMPORTED_ID:0:8}..."
        PASS=$((PASS + 1))
        
        # 18.3 Update Theme (upload new ZIP)
        # Update manifest version for update test
        sed -i.bak 's/"version": "1.0.0"/"version": "1.0.1"/' "$SCRIPT_DIR/fixtures/test-theme-crud/manifest.json"
        cd "$SCRIPT_DIR/fixtures/test-theme-crud" && zip -rq ../test-theme-crud.zip . && cd - > /dev/null
        
        UPDATE_RESP=$(curl -s -X POST "$URL/api/v1/admin/themes/$IMPORTED_ID/update" \
            -H "Authorization: Bearer $TOKEN" \
            -F "file=@$SCRIPT_DIR/fixtures/test-theme-crud.zip")
        
        UPDATE_SUCCESS=$(echo "$UPDATE_RESP" | jq -r '.success // empty')
        
        if [ "$UPDATE_SUCCESS" = "true" ]; then
            echo "✅ POST /admin/themes/{id}/update success = true"
            PASS=$((PASS + 1))
        else
            echo "❌ POST /admin/themes/{id}/update failed"
            FAIL=$((FAIL + 1))
        fi
        
        # 18.4 Delete Theme
        DELETE_RESP=$(curl -s -X DELETE "$URL/api/v1/admin/themes/$IMPORTED_ID" \
            -H "Authorization: Bearer $TOKEN")
        
        DELETE_SUCCESS=$(echo "$DELETE_RESP" | jq -r '.success // empty')
        
        if [ "$DELETE_SUCCESS" = "true" ]; then
            echo "✅ DELETE /admin/themes/{id} success = true"
            PASS=$((PASS + 1))
        else
            echo "❌ DELETE /admin/themes/{id} failed"
            FAIL=$((FAIL + 1))
        fi
        
    else
        echo "❌ POST /admin/themes/import failed (skip update/delete)"
        echo "   Response: $IMPORT_RESP"
        FAIL=$((FAIL + 3))
    fi
fi
echo ""

# ═══════════════════════════════════════════════════════════
# Step 23: CMS CRUD (Phase 2)
# ═══════════════════════════════════════════════════════════
echo "━━━ Step 23: CMS CRUD ━━━"

# 23.1 List CMS Pages
CMS_LIST=$(curl -s -X GET "$URL/api/v1/admin/cms/pages" \
    -H "Authorization: Bearer $TOKEN")

CMS_LIST_SUCCESS=$(echo "$CMS_LIST" | jq -r '.success // empty')
CMS_HAS_META=$(echo "$CMS_LIST" | jq 'has("meta")')

if [ "$CMS_LIST_SUCCESS" = "true" ]; then
    echo "✅ GET /admin/cms/pages success = true"
    PASS=$((PASS + 1))
else
    echo "❌ GET /admin/cms/pages success = $CMS_LIST_SUCCESS"
    FAIL=$((FAIL + 1))
fi

if [ "$CMS_HAS_META" = "true" ]; then
    echo "✅ CMS list has 'meta' field (ApiResponse format)"
    PASS=$((PASS + 1))
else
    echo "❌ CMS list missing 'meta' field"
    FAIL=$((FAIL + 1))
fi

# 23.2 Create CMS Page
CMS_SLUG="test-page-$(date +%s)"
CMS_CREATE=$(curl -s -X POST "$URL/api/v1/admin/cms/pages" \
    -H "Authorization: Bearer $TOKEN" \
    -H "Content-Type: application/json" \
    -d "{\"slug\": \"$CMS_SLUG\", \"title\": \"Test Page\", \"content\": {\"blocks\": []}, \"is_published\": true}")

CMS_CREATE_SUCCESS=$(echo "$CMS_CREATE" | jq -r '.success // empty')
CMS_NEW_ID=$(echo "$CMS_CREATE" | jq -r '.data.id // empty')

if [ "$CMS_CREATE_SUCCESS" = "true" ] && [ -n "$CMS_NEW_ID" ]; then
    echo "✅ POST /admin/cms/pages created ID = ${CMS_NEW_ID:0:8}..."
    PASS=$((PASS + 1))
    
    # 23.3 Get CMS Page
    CMS_GET=$(curl -s -X GET "$URL/api/v1/admin/cms/pages/$CMS_NEW_ID" \
        -H "Authorization: Bearer $TOKEN")
    
    CMS_GET_SUCCESS=$(echo "$CMS_GET" | jq -r '.success // empty')
    
    if [ "$CMS_GET_SUCCESS" = "true" ]; then
        echo "✅ GET /admin/cms/pages/{id} success = true"
        PASS=$((PASS + 1))
    else
        echo "❌ GET /admin/cms/pages/{id} success = $CMS_GET_SUCCESS"
        FAIL=$((FAIL + 1))
    fi
    
    # 23.4 Update CMS Page
    CMS_UPDATE=$(curl -s -X PUT "$URL/api/v1/admin/cms/pages/$CMS_NEW_ID" \
        -H "Authorization: Bearer $TOKEN" \
        -H "Content-Type: application/json" \
        -d "{\"slug\": \"$CMS_SLUG\", \"title\": \"Test Page Updated\", \"content\": {\"blocks\": [\"updated\"]}, \"is_published\": true}")
    
    CMS_UPDATE_SUCCESS=$(echo "$CMS_UPDATE" | jq -r '.success // empty')
    
    if [ "$CMS_UPDATE_SUCCESS" = "true" ]; then
        echo "✅ PUT /admin/cms/pages/{id} success = true"
        PASS=$((PASS + 1))
    else
        echo "❌ PUT /admin/cms/pages/{id} success = $CMS_UPDATE_SUCCESS"
        FAIL=$((FAIL + 1))
    fi
    
    # 23.5 Delete CMS Page
    CMS_DELETE=$(curl -s -X DELETE "$URL/api/v1/admin/cms/pages/$CMS_NEW_ID" \
        -H "Authorization: Bearer $TOKEN")
    
    CMS_DELETE_SUCCESS=$(echo "$CMS_DELETE" | jq -r '.success // empty')
    
    if [ "$CMS_DELETE_SUCCESS" = "true" ]; then
        echo "✅ DELETE /admin/cms/pages/{id} success = true"
        PASS=$((PASS + 1))
    else
        echo "❌ DELETE /admin/cms/pages/{id} success = $CMS_DELETE_SUCCESS"
        FAIL=$((FAIL + 1))
    fi
    
else
    echo "❌ POST /admin/cms/pages failed to create (skip get/update/delete)"
    FAIL=$((FAIL + 4))
fi
echo ""

# ═══════════════════════════════════════════════════════════
# Step 24: Users List & Update (Phase 2)
# ═══════════════════════════════════════════════════════════
echo "━━━ Step 24: Users CRUD ━━━"

# 24.1 List Users
USERS_LIST=$(curl -s -X GET "$URL/api/v1/admin/users" \
    -H "Authorization: Bearer $TOKEN")

USERS_LIST_SUCCESS=$(echo "$USERS_LIST" | jq -r '.success // empty')
USERS_HAS_META=$(echo "$USERS_LIST" | jq 'has("meta")')
USERS_COUNT=$(echo "$USERS_LIST" | jq -r '.data.users | length // 0')

if [ "$USERS_LIST_SUCCESS" = "true" ]; then
    echo "✅ GET /admin/users success = true"
    PASS=$((PASS + 1))
else
    echo "❌ GET /admin/users success = $USERS_LIST_SUCCESS"
    FAIL=$((FAIL + 1))
fi

if [ "$USERS_HAS_META" = "true" ]; then
    echo "✅ Users list has 'meta' field (ApiResponse format)"
    PASS=$((PASS + 1))
else
    echo "❌ Users list missing 'meta' field"
    FAIL=$((FAIL + 1))
fi

if [ "$USERS_COUNT" -ge 1 ]; then
    echo "✅ users count = $USERS_COUNT (>= 1)"
    PASS=$((PASS + 1))
else
    echo "❌ users count = $USERS_COUNT (expected >= 1)"
    FAIL=$((FAIL + 1))
fi

# 24.2 Get first user ID for update test
FIRST_USER_ID=$(echo "$USERS_LIST" | jq -r '.data.users[0].id // empty')

if [ -n "$FIRST_USER_ID" ]; then
    # 24.3 Update User (only update full_name to avoid side effects)
    USERS_UPDATE=$(curl -s -X PATCH "$URL/api/v1/admin/users/$FIRST_USER_ID" \
        -H "Authorization: Bearer $TOKEN" \
        -H "Content-Type: application/json" \
        -d '{"full_name": "Admin User Updated"}')
    
    USERS_UPDATE_SUCCESS=$(echo "$USERS_UPDATE" | jq -r '.success // empty')
    
    if [ "$USERS_UPDATE_SUCCESS" = "true" ]; then
        echo "✅ PATCH /admin/users/{id} success = true"
        PASS=$((PASS + 1))
    else
        echo "❌ PATCH /admin/users/{id} success = $USERS_UPDATE_SUCCESS"
        FAIL=$((FAIL + 1))
    fi
else
    echo "❌ No users found to test update"
    FAIL=$((FAIL + 1))
fi
echo ""

# ═══════════════════════════════════════════════════════════
# Step 25: API Keys CRUD (Phase 2)
# ═══════════════════════════════════════════════════════════
echo "━━━ Step 25: API Keys CRUD ━━━"

# 25.1 List API Keys
APIKEYS_LIST=$(curl -s -X GET "$URL/api/v1/admin/api-keys" \
    -H "Authorization: Bearer $TOKEN")

APIKEYS_LIST_SUCCESS=$(echo "$APIKEYS_LIST" | jq -r '.success // empty')
APIKEYS_HAS_META=$(echo "$APIKEYS_LIST" | jq 'has("meta")')

if [ "$APIKEYS_LIST_SUCCESS" = "true" ]; then
    echo "✅ GET /admin/api-keys success = true"
    PASS=$((PASS + 1))
else
    echo "❌ GET /admin/api-keys success = $APIKEYS_LIST_SUCCESS"
    FAIL=$((FAIL + 1))
fi

if [ "$APIKEYS_HAS_META" = "true" ]; then
    echo "✅ API keys list has 'meta' field (ApiResponse format)"
    PASS=$((PASS + 1))
else
    echo "❌ API keys list missing 'meta' field"
    FAIL=$((FAIL + 1))
fi

# 25.2 Create API Key (need tenant_id from /admin/tenants/me)
TENANT_ME=$(curl -s -X GET "$URL/api/v1/admin/tenants/me" -H "Authorization: Bearer $TOKEN")
TENANT_ID_FOR_KEY=$(echo "$TENANT_ME" | jq -r '.data.id // empty')

if [ -n "$TENANT_ID_FOR_KEY" ]; then
    APIKEY_CREATE=$(curl -s -X POST "$URL/api/v1/admin/api-keys" \
        -H "Authorization: Bearer $TOKEN" \
        -H "Content-Type: application/json" \
        -d "{\"tenant_id\": \"$TENANT_ID_FOR_KEY\", \"name\": \"Test API Key $(date +%s)\", \"key_type\": \"server\"}")
    
    APIKEY_CREATE_SUCCESS=$(echo "$APIKEY_CREATE" | jq -r '.success // empty')
    APIKEY_NEW_ID=$(echo "$APIKEY_CREATE" | jq -r '.data.id // empty')
    APIKEY_PLAIN=$(echo "$APIKEY_CREATE" | jq -r '.data.plain_key // empty')
    
    if [ "$APIKEY_CREATE_SUCCESS" = "true" ] && [ -n "$APIKEY_NEW_ID" ]; then
        echo "✅ POST /admin/api-keys created ID = ${APIKEY_NEW_ID:0:8}..."
        PASS=$((PASS + 1))
        
        if [ -n "$APIKEY_PLAIN" ]; then
            echo "✅ API key plain_key returned (shown once)"
            PASS=$((PASS + 1))
        else
            echo "❌ API key plain_key not returned"
            FAIL=$((FAIL + 1))
        fi
        
        # 25.3 Revoke API Key
        APIKEY_REVOKE=$(curl -s -X DELETE "$URL/api/v1/admin/api-keys/$APIKEY_NEW_ID" \
            -H "Authorization: Bearer $TOKEN")
        
        APIKEY_REVOKE_SUCCESS=$(echo "$APIKEY_REVOKE" | jq -r '.success // empty')
        
        if [ "$APIKEY_REVOKE_SUCCESS" = "true" ]; then
            echo "✅ DELETE /admin/api-keys/{id} success = true"
            PASS=$((PASS + 1))
        else
            echo "❌ DELETE /admin/api-keys/{id} success = $APIKEY_REVOKE_SUCCESS"
            FAIL=$((FAIL + 1))
        fi
    else
        echo "❌ POST /admin/api-keys failed to create (skip revoke)"
        echo "   Response: $(echo "$APIKEY_CREATE" | head -c 200)"
        FAIL=$((FAIL + 3))
    fi
else
    echo "❌ No tenant_id available for API key creation"
    FAIL=$((FAIL + 4))
fi
echo ""

# ═══════════════════════════════════════════════════════════
# Summary
# ═══════════════════════════════════════════════════════════
echo "======================================"
echo "📊 Summary"
echo "======================================"
echo "PASS: $PASS"
echo "FAIL: $FAIL"
echo ""

if [ $FAIL -eq 0 ]; then
    echo "✅ All tests passed!"
    exit 0
else
    echo "❌ Some tests failed"
    exit 1
fi

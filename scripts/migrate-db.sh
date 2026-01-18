#!/bin/bash
# ================================================================
# Kyx Kernel - Database Migration Script
# ================================================================
# Runs database migrations SEPARATELY from the application.
# This script should be run BEFORE starting/updating the app.
#
# Usage:
#   ./scripts/migrate-db.sh              # Run all pending migrations
#   ./scripts/migrate-db.sh --status     # Check migration status
#   ./scripts/migrate-db.sh --rollback   # Rollback last migration
# ================================================================

set -e

SCRIPT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
cd "$SCRIPT_DIR/.."

# Colors
RED='\033[0;31m'
GREEN='\033[0;32m'
YELLOW='\033[1;33m'
BLUE='\033[0;34m'
NC='\033[0m'

log() { echo -e "${BLUE}[MIGRATE]${NC} $1"; }
success() { echo -e "${GREEN}[OK]${NC} $1"; }
warn() { echo -e "${YELLOW}[WARN]${NC} $1"; }
error() { echo -e "${RED}[ERROR]${NC} $1"; exit 1; }

# Load environment variables
if [ -f .env ]; then
    source .env
fi

# Build DATABASE_URL from env or use default
if [ -z "$DATABASE_URL" ]; then
    DB_HOST="${DB_HOST:-localhost}"
    DB_PORT="${DB_PORT:-5435}"
    DB_USER="${DB_USER:-kyx}"
    DB_PASSWORD="${DB_PASSWORD:-kyx_password}"
    DB_NAME="${DB_NAME:-kyx_db}"
    DATABASE_URL="postgres://${DB_USER}:${DB_PASSWORD}@${DB_HOST}:${DB_PORT}/${DB_NAME}"
fi

log "Database URL: ${DATABASE_URL%@*}@****"

# Parse arguments
ACTION="run"
while [[ $# -gt 0 ]]; do
    case $1 in
        --status) ACTION="status"; shift ;;
        --rollback) ACTION="rollback"; shift ;;
        *) shift ;;
    esac
done

# Check for sqlx-cli
if ! command -v sqlx &> /dev/null; then
    warn "sqlx-cli not found. Installing..."
    cargo install sqlx-cli --no-default-features --features postgres
fi

case $ACTION in
    status)
        log "Checking migration status..."
        sqlx migrate info --source ./migrations --database-url "$DATABASE_URL"
        ;;
    rollback)
        log "Rolling back last migration..."
        sqlx migrate revert --source ./migrations --database-url "$DATABASE_URL"
        success "Rollback complete"
        ;;
    run)
        log "Running pending migrations..."
        
        # Count migration files
        TOTAL=$(ls -1 migrations/*.sql 2>/dev/null | wc -l | tr -d ' ')
        log "Found $TOTAL migration files"
        
        # Run migrations
        sqlx migrate run --source ./migrations --database-url "$DATABASE_URL"
        
        success "All migrations applied successfully! ✅"
        ;;
esac

echo ""
log "Migration complete!"

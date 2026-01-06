#!/bin/bash
# ════════════════════════════════════════════════════════════════════════════
# Kyx Kernel Entrypoint Script
# Runs database migrations before starting the application
# ════════════════════════════════════════════════════════════════════════════

set -e

echo "╔══════════════════════════════════════════════════════════════════════╗"
echo "║                    Kyx Kernel - Starting Up                         ║"
echo "╚══════════════════════════════════════════════════════════════════════╝"

# Wait for database to be ready
echo "⏳ Waiting for database..."
until nc -z "${DATABASE_HOST:-localhost}" "${DATABASE_PORT:-5432}"; do
    echo "   Database not ready, retrying in 2s..."
    sleep 2
done
echo "✅ Database is ready!"

# Run migrations if MIGRATE_ON_START is set
if [ "${MIGRATE_ON_START:-true}" = "true" ]; then
    echo "🔄 Running database migrations..."
    
    # Use sqlx-cli if available, otherwise use built-in migrate binary
    if command -v sqlx &> /dev/null; then
        sqlx migrate run
    elif [ -f "/app/migrate" ]; then
        /app/migrate
    else
        echo "⚠️  No migration tool found, skipping migrations"
    fi
    
    echo "✅ Migrations complete!"
fi

echo "🚀 Starting Kyx Kernel..."
exec /app/kyx-kernel "$@"

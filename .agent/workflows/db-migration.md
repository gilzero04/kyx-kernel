# Database Migration Workflow
// turbo-all

This workflow describes how to reset/migrate the database following the **Principle of Least Privilege**.

## Users

| User | Purpose | Permissions |
|------|---------|-------------|
| `postgres` | DBA / Migrations | Superuser (CREATE/DROP DATABASE, etc.) |
| `kyx_admin` | Application Runtime | SELECT, INSERT, UPDATE, DELETE only |

## Reset Database (Development)

1. Stop the Kernel (if running)
```bash
lsof -ti :8080 | xargs kill -9 2>/dev/null
```

2. Rebuild the PostgreSQL container (this recreates the database from scratch)
```bash
cd /Users/beykyu/Developer/kyx-tech/kyx-kernel
docker-compose down
docker-compose build db --no-cache
docker-compose up -d db
```

3. Wait for DB to be ready, then start the Kernel
```bash
sleep 5 && cargo run --bin kyx-kernel
```

## Apply Schema Changes (Without Data Loss)

For production-like migrations where data preservation is needed:
1. Connect as `postgres` superuser (via TablePlus, pgAdmin, etc.)
2. Run ALTER TABLE statements manually
3. Or use a migration tool like `sqlx migrate`

## Notes
- **Never give `kyx_admin` CREATEDB or SUPERUSER** in production
- The Kernel automatically creates tables on startup if they don't exist

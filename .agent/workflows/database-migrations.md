---
description: How to manage and run database migrations in Kyx Kernel
---

## Overview
The Kyx Kernel uses `sqlx` migrations to manage database schema changes safely.

## 1. Creating a New Migration
To add a new schema change, create a new SQL file in the `migrations/` directory:
- **Location**: `migrations/`
- **Naming Pattern**: `[YYYYMMDDHHMMSS]_[description].sql`
- **Example**: `20251225220000_add_user_profile.sql`

## 2. Running Migrations

### Automatic (Recommended)
Migrations run **automatically** every time the Kernel starts. 
Simply restart your application to apply pending changes.

### Manual Execution
If you want to run migrations without starting the full application:
```bash
cargo run --bin migrate
```

## 3. Rolling Back (Native SQLX CLI)
If you have `sqlx-cli` installed, you can also use:
```bash
# Check status
sqlx migrate info

# Run pending
sqlx migrate run
```

> [!IMPORTANT]
> Always check your SQL syntax before adding a migration, as failing migrations will prevent the Kernel from starting.

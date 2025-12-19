use sqlx::PgPool;
use anyhow::Result;

pub async fn init(pool: &PgPool) -> Result<()> {
    // 1. Audit Logs (Transaction table - only created_at)
    sqlx::query(
        r#"
        CREATE TABLE IF NOT EXISTS audit_logs (
            id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
            timestamp TIMESTAMPTZ NOT NULL DEFAULT NOW(),
            actor TEXT NOT NULL,
            action TEXT NOT NULL,
            target TEXT,
            status TEXT NOT NULL,
            metadata JSONB,
            created_at TIMESTAMPTZ DEFAULT NOW()
        )
        "#
    ).execute(pool).await?;

    sqlx::query("CREATE INDEX IF NOT EXISTS idx_audit_logs_timestamp ON audit_logs(timestamp)").execute(pool).await?;
    sqlx::query("CREATE INDEX IF NOT EXISTS idx_audit_logs_actor ON audit_logs(actor)").execute(pool).await?;

    // 2. API Keys (System table - full timestamps)
    sqlx::query(
        "CREATE TABLE IF NOT EXISTS sys_api_keys (
            id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
            tenant_id VARCHAR(255) NOT NULL,
            key_hash VARCHAR(255) NOT NULL,
            prefix VARCHAR(10) NOT NULL,
            name VARCHAR(255),
            key_type VARCHAR(50) DEFAULT 'server',
            allowed_origins JSONB,
            is_active BOOLEAN DEFAULT TRUE,
            created_at TIMESTAMPTZ DEFAULT NOW(),
            updated_at TIMESTAMPTZ DEFAULT NOW(),
            deleted_at TIMESTAMPTZ
        )"
    ).execute(pool).await?;

    // 3. CORS Origins (System table - full timestamps)
    sqlx::query(
        "CREATE TABLE IF NOT EXISTS sys_cors_origins (
            id SERIAL PRIMARY KEY,
            origin VARCHAR(255) UNIQUE NOT NULL,
            is_active BOOLEAN DEFAULT TRUE,
            description TEXT,
            created_at TIMESTAMPTZ DEFAULT NOW(),
            updated_at TIMESTAMPTZ DEFAULT NOW(),
            deleted_at TIMESTAMPTZ
        )"
    ).execute(pool).await?;

    // 4. Apply Triggers (moved to centralized trigger setup)
    init_triggers(pool).await?;

    // Seed Local Platform Origins (Development)
    sqlx::query(
        "INSERT INTO sys_cors_origins (origin, description) VALUES 
         ('http://localhost:5175', 'Local Platform Development'),
         ('http://localhost:5173', 'Vite Default Port'),
         ('http://localhost:4173', 'Vite Preview Port')
         ON CONFLICT (origin) DO NOTHING"
    ).execute(pool).await?;

    Ok(())
}

async fn init_triggers(pool: &PgPool) -> Result<()> {
    // sys_api_keys trigger
    sqlx::query("DROP TRIGGER IF EXISTS update_sys_api_keys_updated_at ON sys_api_keys").execute(pool).await?;
    sqlx::query(
        r#"
        CREATE TRIGGER update_sys_api_keys_updated_at
        BEFORE UPDATE ON sys_api_keys
        FOR EACH ROW EXECUTE FUNCTION update_updated_at_column();
        "#
    ).execute(pool).await?;

    // sys_cors_origins trigger
    sqlx::query("DROP TRIGGER IF EXISTS update_sys_cors_origins_updated_at ON sys_cors_origins").execute(pool).await?;
    sqlx::query(
        r#"
        CREATE TRIGGER update_sys_cors_origins_updated_at
        BEFORE UPDATE ON sys_cors_origins
        FOR EACH ROW EXECUTE FUNCTION update_updated_at_column();
        "#
    ).execute(pool).await?;

    Ok(())
}

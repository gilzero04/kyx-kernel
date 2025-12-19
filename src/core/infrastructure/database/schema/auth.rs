use sqlx::PgPool;
use anyhow::Result;

pub async fn init_core(pool: &PgPool) -> Result<()> {
    // 1. Users (Entity table - full timestamps)
    sqlx::query(
        "CREATE TABLE IF NOT EXISTS auth_users (
            id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
            email VARCHAR(255) UNIQUE NOT NULL,
            hashed_password VARCHAR(255) NOT NULL,
            full_name VARCHAR(255),
            is_active BOOLEAN DEFAULT TRUE,
            created_at TIMESTAMPTZ DEFAULT NOW(),
            updated_at TIMESTAMPTZ DEFAULT NOW(),
            deleted_at TIMESTAMPTZ
        )"
    ).execute(pool).await?;

    // 2. Tenant Types Lookup (for flexibility - can add types without code changes)
    sqlx::query(
        "CREATE TABLE IF NOT EXISTS sys_tenant_types (
            id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
            slug VARCHAR(50) UNIQUE NOT NULL,
            name VARCHAR(255) NOT NULL,
            description TEXT,
            icon VARCHAR(50),
            is_system BOOLEAN DEFAULT FALSE,
            is_active BOOLEAN DEFAULT TRUE,
            created_at TIMESTAMPTZ DEFAULT NOW(),
            updated_at TIMESTAMPTZ DEFAULT NOW(),
            deleted_at TIMESTAMPTZ
        )"
    ).execute(pool).await?;

    // Seed default tenant types
    sqlx::query(
        "INSERT INTO sys_tenant_types (slug, name, description, icon, is_system) VALUES 
         ('standard', 'Standard', 'Default tenant type', 'building', TRUE),
         ('branch', 'Branch', 'Branch office or location', 'store', TRUE),
         ('vendor', 'Vendor', 'Vendor or seller in marketplace', 'shop', TRUE),
         ('supplier', 'Supplier', 'Supplier or provider', 'truck', TRUE)
         ON CONFLICT (slug) DO NOTHING"
    ).execute(pool).await?;

    // 3. Tenants (Entity table - full timestamps, supports hierarchy)
    sqlx::query(
        "CREATE TABLE IF NOT EXISTS auth_tenants (
            id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
            name VARCHAR(255) NOT NULL,
            slug VARCHAR(255) UNIQUE NOT NULL,
            parent_id UUID REFERENCES auth_tenants(id) ON DELETE SET NULL,
            tenant_type_id UUID REFERENCES sys_tenant_types(id),
            is_active BOOLEAN DEFAULT TRUE,
            created_at TIMESTAMPTZ DEFAULT NOW(),
            updated_at TIMESTAMPTZ DEFAULT NOW(),
            deleted_at TIMESTAMPTZ
        )"
    ).execute(pool).await?;

    // Index for fast hierarchical queries
    sqlx::query("CREATE INDEX IF NOT EXISTS idx_tenants_parent_id ON auth_tenants(parent_id)").execute(pool).await?;
    sqlx::query("CREATE INDEX IF NOT EXISTS idx_tenants_type_id ON auth_tenants(tenant_type_id)").execute(pool).await?;

    // Apply Triggers
    init_triggers(pool).await?;

    Ok(())
}

async fn init_triggers(pool: &PgPool) -> Result<()> {
    // auth_users trigger
    sqlx::query("DROP TRIGGER IF EXISTS update_auth_users_updated_at ON auth_users").execute(pool).await?;
    sqlx::query(
        r#"
        CREATE TRIGGER update_auth_users_updated_at
        BEFORE UPDATE ON auth_users
        FOR EACH ROW EXECUTE FUNCTION update_updated_at_column();
        "#
    ).execute(pool).await?;

    // auth_tenants trigger
    sqlx::query("DROP TRIGGER IF EXISTS update_auth_tenants_updated_at ON auth_tenants").execute(pool).await?;
    sqlx::query(
        r#"
        CREATE TRIGGER update_auth_tenants_updated_at
        BEFORE UPDATE ON auth_tenants
        FOR EACH ROW EXECUTE FUNCTION update_updated_at_column();
        "#
    ).execute(pool).await?;

    // sys_tenant_types trigger
    sqlx::query("DROP TRIGGER IF EXISTS update_sys_tenant_types_updated_at ON sys_tenant_types").execute(pool).await?;
    sqlx::query(
        r#"
        CREATE TRIGGER update_sys_tenant_types_updated_at
        BEFORE UPDATE ON sys_tenant_types
        FOR EACH ROW EXECUTE FUNCTION update_updated_at_column();
        "#
    ).execute(pool).await?;

    Ok(())
}

pub async fn init_memberships(pool: &PgPool) -> Result<()> {
    // 3. Memberships (Join table - only created_at)
    sqlx::query(
        "CREATE TABLE IF NOT EXISTS auth_memberships (
            user_id UUID NOT NULL REFERENCES auth_users(id) ON DELETE CASCADE,
            tenant_id UUID NOT NULL REFERENCES auth_tenants(id) ON DELETE CASCADE,
            role VARCHAR(50) NOT NULL,
            role_id UUID REFERENCES sys_roles(id),
            is_active BOOLEAN DEFAULT TRUE,
            created_at TIMESTAMPTZ DEFAULT NOW(),
            PRIMARY KEY (user_id, tenant_id)
        )"
    ).execute(pool).await?;

    Ok(())
}

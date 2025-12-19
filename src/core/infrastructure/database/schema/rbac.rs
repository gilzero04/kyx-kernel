use sqlx::PgPool;
use anyhow::Result;

pub async fn init(pool: &PgPool) -> Result<()> {
    // 1. Tables
    sqlx::query(
        "CREATE TABLE IF NOT EXISTS sys_permissions (
            id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
            code VARCHAR(50) UNIQUE,
            slug VARCHAR(100) UNIQUE NOT NULL,
            name VARCHAR(255) NOT NULL,
            description TEXT,
            sort_order INT DEFAULT 0,
            is_active BOOLEAN DEFAULT TRUE,
            created_at TIMESTAMPTZ DEFAULT NOW(),
            updated_at TIMESTAMPTZ DEFAULT NOW(),
            deleted_at TIMESTAMPTZ
        )"
    ).execute(pool).await?;

    sqlx::query(
        "CREATE TABLE IF NOT EXISTS sys_roles (
            id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
            tenant_id UUID,
            code VARCHAR(50) UNIQUE,
            name VARCHAR(255) NOT NULL,
            slug VARCHAR(100) UNIQUE NOT NULL,
            sort_order INT DEFAULT 0,
            is_active BOOLEAN DEFAULT TRUE,
            created_at TIMESTAMPTZ DEFAULT NOW(),
            updated_at TIMESTAMPTZ DEFAULT NOW(),
            deleted_at TIMESTAMPTZ,
            FOREIGN KEY (tenant_id) REFERENCES auth_tenants(id) ON DELETE CASCADE
        )"
    ).execute(pool).await?;

    sqlx::query(
        "CREATE TABLE IF NOT EXISTS sys_role_permissions (
            role_id UUID REFERENCES sys_roles(id) ON DELETE CASCADE,
            permission_id UUID REFERENCES sys_permissions(id) ON DELETE CASCADE,
            PRIMARY KEY (role_id, permission_id)
        )"
    ).execute(pool).await?;

    // 2. Transact Triggers & Seeding
    init_triggers(pool).await?;
    seed_data(pool).await?;

    Ok(())
}

async fn init_triggers(pool: &PgPool) -> Result<()> {
    // Trigger function already created in schema/mod.rs

    // Apply Triggers for RBAC tables
    sqlx::query("DROP TRIGGER IF EXISTS update_sys_permissions_updated_at ON sys_permissions").execute(pool).await?;
    sqlx::query(
        r#"
        CREATE TRIGGER update_sys_permissions_updated_at
        BEFORE UPDATE ON sys_permissions
        FOR EACH ROW EXECUTE FUNCTION update_updated_at_column();
        "#
    ).execute(pool).await?;

    sqlx::query("DROP TRIGGER IF EXISTS update_sys_roles_updated_at ON sys_roles").execute(pool).await?;
    sqlx::query(
        r#"
        CREATE TRIGGER update_sys_roles_updated_at
        BEFORE UPDATE ON sys_roles
        FOR EACH ROW EXECUTE FUNCTION update_updated_at_column();
        "#
    ).execute(pool).await?;

    Ok(())
}

async fn seed_data(pool: &PgPool) -> Result<()> {
    // 1. Seed Permissions
    let permissions = vec![
        ("P001", "system:manage", "System Management", "Full system control"),
        ("P002", "user:write", "User Management", "Create/Edit users"),
        ("P003", "plugin:install", "Plugin Management", "Install/Update plugins"),
    ];

    for (code, slug, name, desc) in permissions {
        sqlx::query(
            "INSERT INTO sys_permissions (code, slug, name, description) 
             VALUES ($1, $2, $3, $4) ON CONFLICT (slug) DO NOTHING"
        )
        .bind(code).bind(slug).bind(name).bind(desc)
        .execute(pool).await?;
    }

    // 2. Seed Roles
    let roles = vec![
        ("R001", "superadmin", "Super Administrator", 100),
        ("R002", "admin", "Administrator", 80),
        ("R003", "operator", "Operator", 60),
        ("R004", "viewer", "Viewer", 40),
    ];

    for (code, slug, name, sort) in roles {
        sqlx::query(
            "INSERT INTO sys_roles (code, slug, name, sort_order) 
             VALUES ($1, $2, $3, $4) ON CONFLICT (slug) DO NOTHING"
        )
        .bind(code).bind(slug).bind(name).bind(sort)
        .execute(pool).await?;
    }
    
    // 3. Map SuperAdmin to all permissions
    sqlx::query(
        r#"
        INSERT INTO sys_role_permissions (role_id, permission_id)
        SELECT r.id, p.id FROM sys_roles r, sys_permissions p
        WHERE r.slug = 'superadmin'
        ON CONFLICT DO NOTHING
        "#
    ).execute(pool).await?;

    Ok(())
}

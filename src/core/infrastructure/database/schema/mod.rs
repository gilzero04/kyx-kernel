pub mod sys;
pub mod auth;
pub mod rbac;

use sqlx::PgPool;
use anyhow::Result;

/// Initializes all database tables in correct dependency order.
pub async fn initialize_all(pool: &PgPool) -> Result<()> {
    // 0. Create shared trigger function FIRST
    create_trigger_function(pool).await?;

    // 1. System tables (no dependencies)
    sys::init(pool).await?;

    // 2. Auth core tables (users, tenants - no FK dependencies on other modules)
    auth::init_core(pool).await?;

    // 3. RBAC tables (depends on auth_tenants for FK)
    rbac::init(pool).await?;

    // 4. Auth memberships (depends on users, tenants, AND roles)
    auth::init_memberships(pool).await?;

    Ok(())
}

/// Creates the shared timestamp trigger function used by all tables.
async fn create_trigger_function(pool: &PgPool) -> Result<()> {
    sqlx::query(
        r#"
        CREATE OR REPLACE FUNCTION update_updated_at_column()
        RETURNS TRIGGER AS $$
        BEGIN
            NEW.updated_at = NOW();
            RETURN NEW;
        END;
        $$ language 'plpgsql';
        "#
    ).execute(pool).await?;
    Ok(())
}

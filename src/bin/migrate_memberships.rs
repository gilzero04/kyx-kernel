use sqlx::postgres::PgPoolOptions;
use std::env;

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    dotenvy::dotenv().ok();
    let db_url = env::var("DATABASE_URL").expect("DATABASE_URL must be set");
    let pool = PgPoolOptions::new().connect(&db_url).await?;

    println!("🚀 Applying schema migration to auth_memberships...");

    // 1. Add columns
    sqlx::query("ALTER TABLE auth_memberships ADD COLUMN IF NOT EXISTS updated_at TIMESTAMPTZ DEFAULT NOW();")
        .execute(&pool)
        .await?;
    println!("  ✅ Added updated_at column");

    sqlx::query("ALTER TABLE auth_memberships ADD COLUMN IF NOT EXISTS deleted_at TIMESTAMPTZ;")
        .execute(&pool)
        .await?;
    println!("  ✅ Added deleted_at column");

    // 2. Add trigger
    sqlx::query("DROP TRIGGER IF EXISTS update_auth_memberships_updated_at ON auth_memberships;")
        .execute(&pool)
        .await?;
    
    sqlx::query(
        r#"
        CREATE TRIGGER update_auth_memberships_updated_at
        BEFORE UPDATE ON auth_memberships
        FOR EACH ROW EXECUTE FUNCTION update_updated_at_column();
        "#
    )
    .execute(&pool)
    .await?;
    println!("  ✅ Added update_updated_at_column trigger");

    println!("🎉 Migration completed successfully!");
    
    Ok(())
}

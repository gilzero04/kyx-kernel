use sqlx::postgres::PgPoolOptions;
use std::env;

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    dotenvy::dotenv().ok();
    let db_url = env::var("DATABASE_URL").expect("DATABASE_URL must be set");
    let pool = PgPoolOptions::new().connect(&db_url).await?;

    println!("Resetting database...");
    sqlx::query("TRUNCATE auth_users, auth_tenants, auth_memberships CASCADE;")
        .execute(&pool)
        .await?;
    println!("Database reset successful.");

    Ok(())
}

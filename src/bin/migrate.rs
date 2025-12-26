use sqlx::postgres::PgPoolOptions;
use std::env;
use anyhow::Result;

#[tokio::main]
async fn main() -> Result<()> {
    dotenvy::dotenv().ok();
    let db_url = env::var("DATABASE_URL").expect("DATABASE_URL must be set");
    
    println!("🔍 Connecting to database for migrations...");
    let pool = PgPoolOptions::new()
        .max_connections(1)
        .connect(&db_url)
        .await?;

    println!("🚀 Running pending migrations...");
    sqlx::migrate!("./migrations")
        .run(&pool)
        .await?;

    println!("🎉 All migrations applied successfully!");
    
    Ok(())
}

use sqlx::postgres::PgPoolOptions;
use anyhow::Result;
use std::env;
use dotenvy::dotenv;

#[tokio::main]
async fn main() -> Result<()> {
    dotenv().ok();
    
    let database_url = env::var("DATABASE_URL").expect("DATABASE_URL must be set");
    
    println!("🔧 Connecting to database...");
    let pool = PgPoolOptions::new()
        .max_connections(1)
        .connect(&database_url)
        .await?;
    
    println!("🗑️ Dropping all tables...");
    
    // Get all table names
    let tables: Vec<(String,)> = sqlx::query_as(
        "SELECT tablename FROM pg_tables WHERE schemaname = 'public'"
    )
    .fetch_all(&pool)
    .await?;
    
    // Drop each table with CASCADE
    for (table,) in tables {
        println!("  Dropping {}...", table);
        let query = format!("DROP TABLE IF EXISTS \"{}\" CASCADE", table);
        sqlx::query(&query).execute(&pool).await?;
    }
    
    // Drop the trigger function
    sqlx::query("DROP FUNCTION IF EXISTS update_updated_at_column() CASCADE")
        .execute(&pool)
        .await?;
    
    println!("✅ All tables dropped successfully!");
    println!("🚀 Now restart the Kernel to recreate tables with new schema.");
    
    Ok(())
}

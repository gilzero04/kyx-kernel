pub mod schema;

use sqlx::{postgres::PgPoolOptions, PgPool};
use anyhow::Result;

pub struct Database {
    pub pool: PgPool,
}

impl Database {
    pub async fn new(database_url: &str) -> Result<Self> {
        let pool = PgPoolOptions::new()
            .max_connections(10)
            .connect(database_url)
            .await?;
        
        Ok(Self { pool })
    }

    pub async fn initialize_tables(&self) -> Result<()> {
        // Run SQL migrations from ./migrations directory
        // All schema is now defined in migration files
        sqlx::migrate!("./migrations")
            .run(&self.pool)
            .await?;

        log::info!("✅ Database migrations completed successfully.");
        Ok(())
    }
}

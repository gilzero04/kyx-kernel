use anyhow::Result;
use sqlx::{PgPool, postgres::PgPoolOptions};

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

    // Note: Migrations are now run separately via `./scripts/migrate-db.sh`
    // before starting the application. This ensures:
    // 1. Migrations can fail without crashing the app
    // 2. Rollback is possible before app update
    // 3. Zero-downtime updates
}

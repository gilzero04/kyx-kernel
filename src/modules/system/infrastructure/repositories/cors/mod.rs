use crate::modules::system::domain::cors::{CorsOrigin, CorsRepository};
use crate::core::infrastructure::database::Database;
use async_trait::async_trait;
use anyhow::{Result, anyhow};
use std::sync::Arc;

pub struct PostgresCorsRepository {
    pool: Arc<Database>,
}

impl PostgresCorsRepository {
    pub fn new(pool: Arc<Database>) -> Self {
        Self { pool }
    }
}

#[async_trait]
impl CorsRepository for PostgresCorsRepository {
    async fn list(&self) -> Result<Vec<CorsOrigin>> {
        let rows = sqlx::query_as::<_, CorsOrigin>(
            "SELECT id, origin, is_active, description FROM sys_cors_origins ORDER BY created_at DESC"
        )
        .fetch_all(&self.pool.pool)
        .await
        .map_err(|e| anyhow!("Database error: {}", e))?;

        Ok(rows)
    }

    async fn add(&self, origin: &str, description: Option<String>) -> Result<CorsOrigin> {
        let row = sqlx::query_as::<_, CorsOrigin>(
            "INSERT INTO sys_cors_origins (origin, description) VALUES ($1, $2) RETURNING id, origin, is_active, description"
        )
        .bind(origin)
        .bind(description)
        .fetch_one(&self.pool.pool)
        .await
        .map_err(|e| anyhow!("Failed to add origin: {}", e))?;

        Ok(row)
    }

    async fn delete(&self, id: i32) -> Result<()> {
        sqlx::query("DELETE FROM sys_cors_origins WHERE id = $1")
            .bind(id)
            .execute(&self.pool.pool)
            .await
            .map_err(|e| anyhow!("Failed to delete origin: {}", e))?;

        Ok(())
    }
}

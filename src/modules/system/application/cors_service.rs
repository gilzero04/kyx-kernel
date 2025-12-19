use std::sync::Arc;
use crate::core::infrastructure::cors::CorsManager;
use crate::core::infrastructure::database::Database;
use crate::core::AppError;
use serde::{Deserialize, Serialize};
use sqlx::FromRow;

#[derive(Debug, Serialize, Deserialize, FromRow)]
pub struct CorsOrigin {
    pub id: i32,
    pub origin: String,
    pub is_active: Option<bool>,
    pub description: Option<String>,
}

pub struct CORSService {
    db: Arc<Database>,
    manager: Arc<CorsManager>,
}

impl CORSService {
    pub fn new(db: Arc<Database>, manager: Arc<CorsManager>) -> Self {
        Self { db, manager }
    }

    pub async fn list_origins(&self) -> Result<Vec<CorsOrigin>, AppError> {
        let rows = sqlx::query_as::<_, CorsOrigin>(
            "SELECT id, origin, is_active, description FROM sys_cors_origins ORDER BY created_at DESC"
        )
        .fetch_all(&self.db.pool)
        .await?;

        Ok(rows)
    }

    pub async fn add_origin(&self, origin: &str, description: Option<String>) -> Result<CorsOrigin, AppError> {
        let row = sqlx::query_as::<_, CorsOrigin>(
            "INSERT INTO sys_cors_origins (origin, description) VALUES ($1, $2) RETURNING id, origin, is_active, description"
        )
        .bind(origin)
        .bind(description)
        .fetch_one(&self.db.pool)
        .await?;

        self.manager.refresh().await?;
        Ok(row)
    }

    pub async fn delete_origin(&self, id: i32) -> Result<(), AppError> {
        sqlx::query("DELETE FROM sys_cors_origins WHERE id = $1")
            .bind(id)
            .execute(&self.db.pool)
            .await?;

        self.manager.refresh().await?;
        Ok(())
    }
}

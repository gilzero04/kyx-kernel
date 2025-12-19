use std::sync::Arc;
use crate::core::infrastructure::redis::Redis;
use crate::core::infrastructure::database::Database;
use crate::core::AppError;
use sqlx::Row;

pub struct CorsManager {
    redis: Arc<Redis>,
    db: Arc<Database>,
}

impl CorsManager {
    pub fn new(redis: Arc<Redis>, db: Arc<Database>) -> Self {
        Self { redis, db }
    }

    pub async fn refresh(&self) -> Result<(), AppError> {
        let key = std::env::var("REDIS_CORS_KEY").unwrap_or_else(|_| "config:cors_origins".to_string());
        
        // 1. Fetch from Database (exclude soft-deleted)
        let rows = sqlx::query("SELECT origin FROM sys_cors_origins WHERE is_active = TRUE AND deleted_at IS NULL")
            .fetch_all(&self.db.pool)
            .await
            .map_err(|e| AppError {
                code: 500,
                message: format!("Failed to fetch CORS from DB: {}", e),
            })?;
            
        let origins: Vec<String> = rows.iter().map(|r| r.get("origin")).collect();
        
        // 2. Sync to Redis (Transactional)
        let mut conn = self.redis.get_connection();
        
        // Delete old set
        let _: Result<i32, _> = redis::cmd("DEL").arg(&key).query_async(&mut conn).await;
        
        // Add new set if any
        if !origins.is_empty() {
            let mut cmd = redis::cmd("SADD");
            cmd.arg(&key);
            for origin in &origins {
                cmd.arg(origin);
            }
            let _: () = cmd.query_async(&mut conn).await.map_err(|e| AppError {
                code: 500,
                message: format!("Failed to sync CORS to Redis: {}", e),
            })?;
        }
        
        log::info!("✅ CORS Registry synced ({} origins)", origins.len());
        Ok(())
    }

    pub async fn is_origin_allowed(&self, origin: &str) -> bool {
        let key = std::env::var("REDIS_CORS_KEY").unwrap_or_else(|_| "config:cors_origins".to_string());
        let mut conn = self.redis.get_connection();
        
        let is_member: bool = redis::cmd("SISMEMBER")
            .arg(&key)
            .arg(origin)
            .query_async(&mut conn)
            .await
            .unwrap_or(false);
            
        is_member
    }
}

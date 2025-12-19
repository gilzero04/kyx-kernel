use std::sync::Arc;
use crate::core::infrastructure::redis::Redis;
use crate::core::AppError;
use dashmap::DashMap;
use std::time::{Duration, Instant};
use serde_json::Value;

pub struct ConfigService {
    db: Arc<crate::core::infrastructure::database::Database>,
    redis: Arc<Redis>,
    cache: DashMap<String, (Value, Instant)>,
    ttl: Duration,
}

impl ConfigService {
    pub fn new(
        db: Arc<crate::core::infrastructure::database::Database>,
        redis: Arc<Redis>
    ) -> Self {
        Self {
            db,
            redis,
            cache: DashMap::new(),
            ttl: Duration::from_secs(60), // Cache for 60 seconds
        }
    }

    pub async fn get_string(&self, key: &str, default: &str) -> String {
        if let Some(val) = self.get_cached_value(key) {
            return val.as_str().unwrap_or(default).to_string();
        }

        let redis_key = format!("config:{}", key);
        let mut conn = self.redis.get_connection();
        
        let val: Option<String> = redis::cmd("GET")
            .arg(&redis_key)
            .query_async(&mut conn)
            .await
            .unwrap_or(None);

        if let Some(s) = val {
            let json_val = Value::from(s.clone());
            self.cache.insert(key.to_string(), (json_val, Instant::now()));
            return s;
        }

        // Try PostgreSQL fallback
        if let Some(db_val) = self.get_from_db(key).await {
            if let Some(s) = db_val.as_str() {
                let s_str = s.to_string();
                self.cache.insert(key.to_string(), (db_val, Instant::now()));
                // Sync back to Redis for future hits
                let mut conn = self.redis.get_connection();
                let _ : () = redis::cmd("SET").arg(format!("config:{}", key)).arg(&s_str).query_async(&mut conn).await.unwrap_or(());
                return s_str;
            }
        }

        // Fallback to Env or default
        let env_val = std::env::var(key.to_uppercase())
            .unwrap_or_else(|_| default.to_string());
            
        env_val
    }

    pub async fn get_int(&self, key: &str, default: i64) -> i64 {
        if let Some(val) = self.get_cached_value(key) {
            return val.as_i64().unwrap_or(default);
        }

        let redis_key = format!("config:{}", key);
        let mut conn = self.redis.get_connection();
        
        let val: Option<String> = redis::cmd("GET")
            .arg(&redis_key)
            .query_async(&mut conn)
            .await
            .unwrap_or(None);

        if let Some(s) = val {
            if let Ok(i) = s.parse::<i64>() {
                let json_val = Value::from(i);
                self.cache.insert(key.to_string(), (json_val, Instant::now()));
                return i;
            }
        }

        // Try PostgreSQL fallback
        if let Some(db_val) = self.get_from_db(key).await {
            if let Some(i) = db_val.as_i64() {
                self.cache.insert(key.to_string(), (db_val, Instant::now()));
                // Sync back to Redis
                let mut conn = self.redis.get_connection();
                let _ : () = redis::cmd("SET").arg(format!("config:{}", key)).arg(i.to_string()).query_async(&mut conn).await.unwrap_or(());
                return i;
            }
        }

        // Fallback to Env or default
        let env_val = std::env::var(key.to_uppercase())
            .ok()
            .and_then(|v| v.parse::<i64>().ok())
            .unwrap_or(default);
            
        env_val
    }

    pub async fn set(&self, key: &str, value: Value) -> Result<(), AppError> {
        // 1. Persistence - Update PostgreSQL first
        sqlx::query(
            "INSERT INTO sys_configs (key, value) VALUES ($1, $2)
             ON CONFLICT (key) DO UPDATE SET value = EXCLUDED.value"
        )
        .bind(key)
        .bind(&value)
        .execute(&self.db.pool)
        .await
        .map_err(|e| AppError {
            code: 500,
            message: format!("Failed to update config in Postgres: {}", e),
        })?;

        // 2. Cache - Update Redis (use raw string for strings, JSON for complex types)
        let redis_key = format!("config:{}", key);
        let mut conn = self.redis.get_connection();
        let val_str = match &value {
            Value::String(s) => s.clone(),
            _ => value.to_string(),
        };
        
        let _: () = redis::cmd("SET")
            .arg(&redis_key)
            .arg(&val_str)
            .query_async(&mut conn)
            .await
            .map_err(|e| AppError {
                code: 500,
                message: format!("Failed to update config in Redis: {}", e),
            })?;

        // 3. Invalidate local memory cache
        self.cache.remove(key);
        
        Ok(())
    }

    async fn get_from_db(&self, key: &str) -> Option<Value> {
        let row: Option<(Value,)> = sqlx::query_as("SELECT value FROM sys_configs WHERE key = $1")
            .bind(key)
            .fetch_optional(&self.db.pool)
            .await
            .unwrap_or(None);
        
        row.map(|(v,)| v)
    }

    fn get_cached_value(&self, key: &str) -> Option<Value> {
        if let Some(entry) = self.cache.get(key) {
            let (val, timestamp) = entry.value();
            if timestamp.elapsed() < self.ttl {
                return Some(val.clone());
            }
        }
        None
    }
}

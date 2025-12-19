use std::sync::Arc;
use crate::core::infrastructure::redis::Redis;
use crate::core::AppError;
use dashmap::DashMap;
use tokio::sync::RwLock;
use std::time::{Duration, Instant};
use serde_json::Value;

pub struct ConfigService {
    redis: Arc<Redis>,
    cache: DashMap<String, (Value, Instant)>,
    ttl: Duration,
}

impl ConfigService {
    pub fn new(redis: Arc<Redis>) -> Self {
        Self {
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

        // Fallback to Env or default
        let env_val = std::env::var(key.to_uppercase())
            .ok()
            .and_then(|v| v.parse::<i64>().ok())
            .unwrap_or(default);
            
        env_val
    }

    pub async fn set(&self, key: &str, value: Value) -> Result<(), AppError> {
        let redis_key = format!("config:{}", key);
        let mut conn = self.redis.get_connection();
        
        let val_str = value.to_string();
        
        let _: () = redis::cmd("SET")
            .arg(&redis_key)
            .arg(&val_str)
            .query_async(&mut conn)
            .await
            .map_err(|e| AppError {
                code: 500,
                message: format!("Failed to update config in Redis: {}", e),
            })?;

        // Invalidate local cache
        self.cache.remove(key);
        
        Ok(())
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

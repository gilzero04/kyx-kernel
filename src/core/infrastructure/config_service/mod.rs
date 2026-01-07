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
        self.get_tenant_string(None, key, default).await
    }

    pub async fn get_tenant_string(&self, tenant_id: Option<uuid::Uuid>, key: &str, default: &str) -> String {
        // 1. Try Tenant Workspace Scope
        if let Some(tid) = tenant_id {
            if let Some(val) = self.find_config_entry(Some(tid), key, Some("workspace")).await {
                return val.as_str().unwrap_or(default).to_string();
            }
        }

        // 2. Try Global Workspace Default (Owner's workspace scope)
        if let Ok(owner_id) = self.get_owner_id().await {
            if let Some(val) = self.find_config_entry(Some(owner_id), key, Some("workspace")).await {
                return val.as_str().unwrap_or(default).to_string();
            }
            
            // 3. Try Platform Scope
            if let Some(val) = self.find_config_entry(Some(owner_id), key, Some("platform")).await {
                return val.as_str().unwrap_or(default).to_string();
            }

            // 4. Try System Scope
            if let Some(val) = self.find_config_entry(Some(owner_id), key, Some("system")).await {
                return val.as_str().unwrap_or(default).to_string();
            }
        }

        // 5. Fallback to Env or default
        std::env::var(key.to_uppercase())
            .unwrap_or_else(|_| default.to_string())
    }

    pub async fn get_int(&self, key: &str, default: i64) -> i64 {
        self.get_tenant_int(None, key, default).await
    }

    pub async fn get_tenant_int(&self, tenant_id: Option<uuid::Uuid>, key: &str, default: i64) -> i64 {
        // 1. Try Tenant Workspace Scope
        if let Some(tid) = tenant_id {
            if let Some(val) = self.find_config_entry(Some(tid), key, Some("workspace")).await {
                return val.as_i64().unwrap_or(default);
            }
        }

        // 2. Try Global Workspace Default (Owner's workspace scope)
        if let Ok(owner_id) = self.get_owner_id().await {
            if let Some(val) = self.find_config_entry(Some(owner_id), key, Some("workspace")).await {
                return val.as_i64().unwrap_or(default);
            }
            
            // 3. Try Platform Scope
            if let Some(val) = self.find_config_entry(Some(owner_id), key, Some("platform")).await {
                return val.as_i64().unwrap_or(default);
            }

            // 4. Try System Scope
            if let Some(val) = self.find_config_entry(Some(owner_id), key, Some("system")).await {
                return val.as_i64().unwrap_or(default);
            }
        }

        // 5. Fallback to Env or default
        std::env::var(key.to_uppercase())
            .ok()
            .and_then(|v| v.parse::<i64>().ok())
            .unwrap_or(default)
    }

    /// Helper to find config for a SPECIFIC tenant scope (or strictly None)
    /// Does not recurse or fallback. Checks L1(Mem) -> L2(Redis) -> L3(DB)
    async fn find_config_entry(&self, tenant_id: Option<uuid::Uuid>, key: &str, scope: Option<&str>) -> Option<Value> {
        let scp = scope.unwrap_or("platform");
        let cache_key = if let Some(tid) = tenant_id {
            format!("{}:{}:{}", tid, scp, key)
        } else {
            // If strictly no tenant_id provided, default to Owner ID fallback
            if let Ok(owner_id) = self.get_owner_id().await {
                format!("{}:{}:{}", owner_id, scp, key)
            } else {
                format!("{}:{}", scp, key)
            }
        };

        // L1: Memory
        if let Some(entry) = self.cache.get(&cache_key) {
            let (val, timestamp) = entry.value();
            if timestamp.elapsed() < self.ttl {
                return Some(val.clone());
            }
        }

        // L2: Redis
        let redis_key = format!("config:{}", cache_key);
        let mut conn = self.redis.get_connection();
        let val_str: Option<String> = redis::cmd("GET")
            .arg(&redis_key)
            .query_async(&mut conn)
            .await
            .unwrap_or(None);

        if let Some(s) = val_str {
            let json_val = serde_json::from_str(&s).unwrap_or_else(|_| Value::from(s.clone()));
            self.cache.insert(cache_key.clone(), (json_val.clone(), Instant::now()));
            return Some(json_val);
        }

        // L3: PostgreSQL
        let effective_tid = if let Some(tid) = tenant_id {
            Some(tid)
        } else {
            self.get_owner_id().await.ok()
        };

        if let Some(tid) = effective_tid {
            if let Some(db_val) = self.get_from_db(tid, key, scp).await {
                 self.cache.insert(cache_key.clone(), (db_val.clone(), Instant::now()));
                 
                 let redis_val = match &db_val {
                     Value::String(s) => s.clone(),
                     _ => db_val.to_string()
                 };
                 
                 let _ : () = redis::cmd("SET").arg(&redis_key).arg(&redis_val).query_async(&mut conn).await.unwrap_or(());
                 return Some(db_val);
            }
        }

        None
    }

    pub async fn set(&self, key: &str, value: Value) -> Result<(), AppError> {
        self.set_tenant_config(None, key, value, None).await
    }

    pub async fn set_tenant_config(&self, tenant_id: Option<uuid::Uuid>, key: &str, value: Value, scope: Option<&str>) -> Result<(), AppError> {
        let tid = if let Some(id) = tenant_id {
            id
        } else {
            self.get_owner_id().await.map_err(|e| AppError {
                code: 500,
                message: format!("Cannot set config without tenant context: {}", e.message),
            })?
        };

        // Determine scope automatically if not specified? 
        // For now, let's use a heuristic or just default to workspace if not platform-like
        let scope = if let Some(s) = scope {
            s
        } else if key.starts_with("console_") || 
                  key.starts_with("access_token") || 
                  key.starts_with("refresh_token") || 
                  key.starts_with("branding_") || 
                  key.starts_with("platform_") || 
                  key.starts_with("theme_") {
            "platform"
        } else {
            "workspace"
        };

        let cache_key = format!("{}:{}:{}", tid, scope, key);
        let redis_key = format!("config:{}", cache_key);

        sqlx::query(
            "INSERT INTO sys_configs (key, value, tenant_id, scope) VALUES ($1, $2, $3, $4)
             ON CONFLICT (key, tenant_id, scope)
             DO UPDATE SET value = EXCLUDED.value"
        )
        .bind(key)
        .bind(&value)
        .bind(tid)
        .bind(scope)
        .execute(&self.db.pool)
        .await
        .map_err(|e| AppError {
            code: 500,
            message: format!("Failed to update config in Postgres: {}", e),
        })?;

        // 2. Cache - Update Redis
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
        self.cache.remove(&cache_key);
        
        Ok(())
    }

    async fn get_from_db(&self, tenant_id: uuid::Uuid, key: &str, scope: &str) -> Option<Value> {
        let row: Option<(Value,)> = sqlx::query_as::<_, (Value,)>(
            "SELECT value FROM sys_configs WHERE key = $1 AND tenant_id = $2 AND scope = $3"
        )
        .bind(key)
        .bind(tenant_id)
        .bind(scope)
        .fetch_optional(&self.db.pool)
        .await
        .unwrap_or(None);
        
        row.map(|(v,)| v)
    }

    pub async fn get_owner_id(&self) -> Result<uuid::Uuid, AppError> {
        // Try to get from cache first?
        let owner_query = "SELECT id FROM auth_tenants WHERE parent_id = id LIMIT 1";
        let owner_id: Option<uuid::Uuid> = sqlx::query_scalar(owner_query)
            .fetch_optional(&self.db.pool)
            .await
            .unwrap_or(None);
        
        owner_id.ok_or_else(|| AppError {
            code: 500,
            message: "System Owner not found".to_string(),
        })
    }
}

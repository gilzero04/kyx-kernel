use std::sync::Arc;
use crate::core::infrastructure::database::Database;
use crate::core::utils::password::{hash_password, verify_password};
use crate::core::AppError;
use serde::{Deserialize, Serialize};
use serde_json::Value;
use uuid::Uuid;
use sqlx::Row;

#[derive(Debug, Serialize, Deserialize)]
pub struct ApiKey {
    pub id: Uuid,
    pub tenant_id: String,
    pub prefix: String,
    pub name: Option<String>,
    pub key_type: String,
    pub allowed_origins: Option<Value>,
    pub is_active: bool,
}

pub struct ApiKeyService {
    db: Arc<Database>,
}

impl ApiKeyService {
    pub fn new(db: Arc<Database>) -> Self {
        Self { db }
    }

    pub async fn create_key(
        &self,
        tenant_id: &str,
        name: Option<String>,
        key_type: &str,
        allowed_origins: Option<Value>,
    ) -> Result<(ApiKey, String), AppError> {
        let plain_key = format!("sk_live_{}", Uuid::new_v4().to_string().replace("-", ""));
        let key_hash = hash_password(&plain_key)?;
        let prefix = &plain_key[..10]; // sk_live_...

        let row = sqlx::query(
            "INSERT INTO sys_api_keys (tenant_id, key_hash, prefix, name, key_type, allowed_origins)
             VALUES ($1, $2, $3, $4, $5, $6)
             RETURNING id, tenant_id, prefix, name, key_type, allowed_origins, is_active"
        )
        .bind(tenant_id)
        .bind(key_hash)
        .bind(prefix)
        .bind(name)
        .bind(key_type)
        .bind(allowed_origins)
        .fetch_one(&self.db.pool)
        .await
        .map_err(|e| AppError {
            code: 500,
            message: format!("Failed to create API key: {}", e),
        })?;

        let key = ApiKey {
            id: row.get("id"),
            tenant_id: row.get("tenant_id"),
            prefix: row.get("prefix"),
            name: row.get("name"),
            key_type: row.get("key_type"),
            allowed_origins: row.get("allowed_origins"),
            is_active: row.get("is_active"),
        };

        Ok((key, plain_key))
    }

    pub async fn verify_key(&self, plain_key: &str, user_agent: Option<&str>, fetch_mode: Option<&str>) -> Result<ApiKey, AppError> {
        if !plain_key.starts_with("sk_live_") {
            return Err(AppError { code: 401, message: "Invalid key format".into() });
        }

        let prefix = &plain_key[..10];
        
        // Find candidates by prefix
        let rows = sqlx::query(
            "SELECT * FROM sys_api_keys WHERE prefix = $1 AND is_active = TRUE"
        )
        .bind(prefix)
        .fetch_all(&self.db.pool)
        .await
        .map_err(|e| AppError { code: 500, message: e.to_string() })?;

        for row in rows {
            let key_hash: String = row.get("key_hash");
            if verify_password(plain_key, &key_hash).unwrap_or(false) {
                let key_type: String = row.get("key_type");
                
                // Security Constraints
                if key_type == "server" {
                    // Block Browser User-Agents
                    if let Some(ua) = user_agent {
                        let ua_lower = ua.to_lowercase();
                        if ua_lower.contains("mozilla") || fetch_mode.is_some() {
                            return Err(AppError {
                                code: 403,
                                message: "Server API Key cannot be used in a browser".into(),
                            });
                        }
                    }
                }

                return Ok(ApiKey {
                    id: row.get("id"),
                    tenant_id: row.get("tenant_id"),
                    prefix: row.get("prefix"),
                    name: row.get("name"),
                    key_type,
                    allowed_origins: row.get("allowed_origins"),
                    is_active: row.get("is_active"),
                });
            }
        }

        Err(AppError { code: 401, message: "Invalid API key".into() })
    }
}

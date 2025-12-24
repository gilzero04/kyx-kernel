use crate::modules::system::domain::api_key::{ApiKey, ApiKeyRepository};
use crate::core::infrastructure::database::Database;
use async_trait::async_trait;
use anyhow::{Result, anyhow};
use std::sync::Arc;
use serde_json::Value;

pub struct PostgresApiKeyRepository {
    pool: Arc<Database>,
}

impl PostgresApiKeyRepository {
    pub fn new(pool: Arc<Database>) -> Self {
        Self { pool }
    }
}

#[async_trait]
impl ApiKeyRepository for PostgresApiKeyRepository {
    async fn create(&self, tenant_id: &str, key_hash: &str, prefix: &str, name: Option<String>, key_type: &str, allowed_origins: Option<Value>) -> Result<ApiKey> {
        let row = sqlx::query_as::<_, ApiKey>(
            "INSERT INTO sys_api_keys (tenant_id, key_hash, prefix, name, key_type, allowed_origins)
             VALUES ($1, $2, $3, $4, $5, $6)
             RETURNING id, tenant_id, prefix, name, key_type, allowed_origins, is_active, key_hash"
        )
        .bind(tenant_id)
        .bind(key_hash)
        .bind(prefix)
        .bind(name)
        .bind(key_type)
        .bind(allowed_origins)
        .fetch_one(&self.pool.pool)
        .await
        .map_err(|e| anyhow!("Failed to create API key: {}", e))?;

        Ok(row)
    }

    async fn find_by_prefix(&self, prefix: &str) -> Result<Vec<ApiKey>> {
        let rows = sqlx::query_as::<_, ApiKey>(
            "SELECT * FROM sys_api_keys WHERE prefix = $1 AND is_active = TRUE"
        )
        .bind(prefix)
        .fetch_all(&self.pool.pool)
        .await
        .map_err(|e| anyhow!("Database error: {}", e))?;

        Ok(rows)
    }
}

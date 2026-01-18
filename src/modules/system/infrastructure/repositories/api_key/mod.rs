use crate::core::infrastructure::database::Database;
use crate::modules::system::domain::api_key::{ApiKey, ApiKeyRepository};
use anyhow::{Result, anyhow};
use async_trait::async_trait;
use serde_json::Value;
use std::sync::Arc;
use uuid::Uuid;

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
    async fn create(
        &self,
        tenant_id: Uuid,
        key_hash: &str,
        prefix: &str,
        name: Option<String>,
        key_type: &str,
        allowed_origins: Option<Value>,
    ) -> Result<ApiKey> {
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

    #[allow(dead_code)]
    async fn find_by_prefix(&self, prefix: &str) -> Result<Vec<ApiKey>> {
        let rows = sqlx::query_as::<_, ApiKey>(
            "SELECT id, tenant_id, prefix, name, key_type, allowed_origins, is_active, key_hash 
             FROM sys_api_keys WHERE prefix = $1 AND is_active = TRUE AND deleted_at IS NULL",
        )
        .bind(prefix)
        .fetch_all(&self.pool.pool)
        .await
        .map_err(|e| anyhow!("Database error: {}", e))?;

        Ok(rows)
    }

    async fn find_hierarchical(&self, tenant_id: Uuid) -> Result<Vec<ApiKey>> {
        let rows = sqlx::query_as::<_, ApiKey>(
            "WITH RECURSIVE tenant_hierarchy AS (
                SELECT id FROM auth_tenants WHERE id = $1
                UNION ALL
                SELECT t.id FROM auth_tenants t
                INNER JOIN tenant_hierarchy th ON t.parent_id = th.id
             )
             SELECT k.id, k.tenant_id, k.prefix, k.name, k.key_type, k.allowed_origins, k.is_active, k.key_hash
             FROM sys_api_keys k
             INNER JOIN tenant_hierarchy th ON k.tenant_id = th.id
             WHERE k.deleted_at IS NULL"
        )
        .bind(tenant_id)
        .fetch_all(&self.pool.pool)
        .await
        .map_err(|e| anyhow!("Failed to fetch hierarchical API keys: {}", e))?;

        Ok(rows)
    }

    async fn delete(&self, key_id: Uuid, actor_tenant_id: Uuid) -> Result<()> {
        let result = sqlx::query(
            "WITH RECURSIVE tenant_hierarchy AS (
                SELECT id FROM auth_tenants WHERE id = $2
                UNION ALL
                SELECT t.id FROM auth_tenants t
                INNER JOIN tenant_hierarchy th ON t.parent_id = th.id
             )
             UPDATE sys_api_keys SET deleted_at = NOW(), is_active = FALSE
             WHERE id = $1 
             AND tenant_id IN (SELECT id FROM tenant_hierarchy)
             AND deleted_at IS NULL",
        )
        .bind(key_id)
        .bind(actor_tenant_id)
        .execute(&self.pool.pool)
        .await
        .map_err(|e| anyhow!("Failed to revoke API key: {}", e))?;

        if result.rows_affected() == 0 {
            return Err(anyhow!("API key not found or access denied"));
        }

        Ok(())
    }
}

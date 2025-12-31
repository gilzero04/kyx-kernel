use super::entity::ApiKey;
use anyhow::Result;
use async_trait::async_trait;
use serde_json::Value;
use uuid::Uuid;

#[async_trait]
pub trait ApiKeyRepository: Send + Sync {
    async fn create(&self, tenant_id: Uuid, key_hash: &str, prefix: &str, name: Option<String>, key_type: &str, allowed_origins: Option<Value>) -> Result<ApiKey>;
    #[allow(dead_code)]
    async fn find_by_prefix(&self, prefix: &str) -> Result<Vec<ApiKey>>;
    async fn find_hierarchical(&self, tenant_id: Uuid) -> Result<Vec<ApiKey>>;
    async fn delete(&self, key_id: Uuid, actor_tenant_id: Uuid) -> Result<()>;
}

use super::entity::ApiKey;
use anyhow::Result;
use async_trait::async_trait;
use serde_json::Value;

#[async_trait]
pub trait ApiKeyRepository: Send + Sync {
    async fn create(&self, tenant_id: &str, key_hash: &str, prefix: &str, name: Option<String>, key_type: &str, allowed_origins: Option<Value>) -> Result<ApiKey>;
    async fn find_by_prefix(&self, prefix: &str) -> Result<Vec<ApiKey>>;
}

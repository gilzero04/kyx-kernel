use super::entity::CorsOrigin;
use anyhow::Result;
use async_trait::async_trait;

#[async_trait]
pub trait CorsRepository: Send + Sync {
    async fn list(&self) -> Result<Vec<CorsOrigin>>;
    async fn add(&self, origin: &str, description: Option<String>) -> Result<CorsOrigin>;
    async fn update(&self, id: i32, is_active: Option<bool>, description: Option<String>) -> Result<CorsOrigin>;
    async fn delete(&self, id: i32) -> Result<()>;
}

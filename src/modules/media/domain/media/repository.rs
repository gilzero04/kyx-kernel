use crate::modules::media::domain::media::entity::{MediaAsset, MediaFolder};
use anyhow::Result;
use async_trait::async_trait;
use uuid::Uuid;

#[async_trait]
pub trait MediaRepository: Send + Sync {
    async fn create_folder(&self, tenant_id: Uuid, parent_id: Option<Uuid>, name: String) -> Result<MediaFolder>;
    async fn list_folders(&self, tenant_id: Uuid, parent_id: Option<Uuid>) -> Result<Vec<MediaFolder>>;
    
    async fn create_asset(&self, tenant_id: Uuid, folder_id: Option<Uuid>, filename: String, original_name: String, mime_type: String, file_size: i64, url: String) -> Result<MediaAsset>;
    async fn list_assets(&self, tenant_id: Uuid, folder_id: Option<Uuid>) -> Result<Vec<MediaAsset>>;
    async fn delete_asset(&self, id: Uuid, tenant_id: Uuid) -> Result<()>;
}

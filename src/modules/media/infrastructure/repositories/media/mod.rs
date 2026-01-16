use crate::core::infrastructure::database::Database;
use crate::modules::media::domain::media::entity::{MediaAsset, MediaFolder};
use crate::modules::media::domain::media::repository::MediaRepository;
use anyhow::Result;
use async_trait::async_trait;
use std::sync::Arc;
use uuid::Uuid;

pub struct PostgresMediaRepository {
    pool: Arc<Database>,
}

impl PostgresMediaRepository {
    pub fn new(pool: Arc<Database>) -> Self {
        Self { pool }
    }
}

#[async_trait]
impl MediaRepository for PostgresMediaRepository {
    async fn create_folder(&self, tenant_id: Uuid, parent_id: Option<Uuid>, name: String) -> Result<MediaFolder> {
        let folder = sqlx::query_as::<_, MediaFolder>(
            "INSERT INTO media_folders (tenant_id, parent_id, name)
             VALUES ($1, $2, $3)
             RETURNING id, tenant_id, parent_id, name, created_at, updated_at"
        )
        .bind(tenant_id)
        .bind(parent_id)
        .bind(name)
        .fetch_one(&self.pool.pool)
        .await?;
        
        Ok(folder)
    }

    async fn list_folders(&self, tenant_id: Uuid, parent_id: Option<Uuid>) -> Result<Vec<MediaFolder>> {
        let folders = sqlx::query_as::<_, MediaFolder>(
            "SELECT id, tenant_id, parent_id, name, created_at, updated_at 
             FROM media_folders 
             WHERE tenant_id = $1 AND (parent_id = $2 OR (parent_id IS NULL AND $2 IS NULL))"
        )
        .bind(tenant_id)
        .bind(parent_id)
        .fetch_all(&self.pool.pool)
        .await?;
        
        Ok(folders)
    }

    async fn create_asset(&self, tenant_id: Uuid, folder_id: Option<Uuid>, filename: String, original_name: String, mime_type: String, file_size: i64, url: String) -> Result<MediaAsset> {
        let asset = sqlx::query_as::<_, MediaAsset>(
            "INSERT INTO media_assets (tenant_id, folder_id, filename, original_name, mime_type, file_size, url)
             VALUES ($1, $2, $3, $4, $5, $6, $7)
             RETURNING id, tenant_id, folder_id, filename, original_name, mime_type, file_size, url, metadata, created_at, updated_at"
        )
        .bind(tenant_id)
        .bind(folder_id)
        .bind(filename)
        .bind(original_name)
        .bind(mime_type)
        .bind(file_size)
        .bind(url)
        .fetch_one(&self.pool.pool)
        .await?;
        
        Ok(asset)
    }

    async fn list_assets(&self, tenant_id: Uuid, folder_id: Option<Uuid>) -> Result<Vec<MediaAsset>> {
        // Include own assets + shared assets (via is_shared broadcast or explicit shares)
        let assets = sqlx::query_as::<_, MediaAsset>(
            "SELECT id, tenant_id, folder_id, filename, original_name, mime_type, file_size, url, metadata, created_at, updated_at 
             FROM media_assets 
             WHERE deleted_at IS NULL AND (folder_id = $2 OR (folder_id IS NULL AND $2 IS NULL)) AND (
                 tenant_id = $1
                 OR can_access_shared_resource('media', id, $1)
             )"
        )
        .bind(tenant_id)
        .bind(folder_id)
        .fetch_all(&self.pool.pool)
        .await?;
        
        Ok(assets)
    }

    async fn delete_asset(&self, id: Uuid, tenant_id: Uuid) -> Result<()> {
        sqlx::query("UPDATE media_assets SET deleted_at = NOW() WHERE id = $1 AND tenant_id = $2")
            .bind(id)
            .bind(tenant_id)
            .execute(&self.pool.pool)
            .await?;
            
        Ok(())
    }
}

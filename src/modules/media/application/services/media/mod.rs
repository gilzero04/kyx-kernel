use crate::core::AppError;
use crate::modules::media::domain::media::entity::{MediaAsset, MediaFolder};
use crate::modules::media::domain::media::repository::MediaRepository;
use std::sync::Arc;
use uuid::Uuid;

pub struct MediaService {
    repo: Arc<dyn MediaRepository>,
}

impl MediaService {
    pub fn new(repo: Arc<dyn MediaRepository>) -> Self {
        Self { repo }
    }

    pub async fn create_folder(
        &self,
        tenant_id: Uuid,
        parent_id: Option<Uuid>,
        name: String,
    ) -> Result<MediaFolder, AppError> {
        self.repo
            .create_folder(tenant_id, parent_id, name)
            .await
            .map_err(|e| AppError {
                code: 500,
                message: e.to_string(),
            })
    }

    pub async fn list_folders(
        &self,
        tenant_id: Uuid,
        parent_id: Option<Uuid>,
    ) -> Result<Vec<MediaFolder>, AppError> {
        self.repo
            .list_folders(tenant_id, parent_id)
            .await
            .map_err(|e| AppError {
                code: 500,
                message: e.to_string(),
            })
    }

    pub async fn create_asset(
        &self,
        tenant_id: Uuid,
        folder_id: Option<Uuid>,
        filename: String,
        original_name: String,
        mime_type: String,
        file_size: i64,
        url: String,
    ) -> Result<MediaAsset, AppError> {
        self.repo
            .create_asset(
                tenant_id,
                folder_id,
                filename,
                original_name,
                mime_type,
                file_size,
                url,
            )
            .await
            .map_err(|e| AppError {
                code: 500,
                message: e.to_string(),
            })
    }

    pub async fn list_assets(
        &self,
        tenant_id: Uuid,
        folder_id: Option<Uuid>,
    ) -> Result<Vec<MediaAsset>, AppError> {
        self.repo
            .list_assets(tenant_id, folder_id)
            .await
            .map_err(|e| AppError {
                code: 500,
                message: e.to_string(),
            })
    }

    pub async fn delete_asset(&self, id: Uuid, tenant_id: Uuid) -> Result<(), AppError> {
        self.repo
            .delete_asset(id, tenant_id)
            .await
            .map_err(|e| AppError {
                code: 500,
                message: e.to_string(),
            })
    }
}

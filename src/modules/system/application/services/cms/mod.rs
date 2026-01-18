use crate::core::AppError;
use crate::modules::system::domain::cms::entity::PageEntry;
use crate::modules::system::infrastructure::repositories::cms::PostgresCmsRepository;
use anyhow::Result;
use std::sync::Arc;
use uuid::Uuid;

pub struct CmsService {
    repo: Arc<PostgresCmsRepository>,
}

impl CmsService {
    pub fn new(repo: Arc<PostgresCmsRepository>) -> Self {
        Self { repo }
    }

    pub async fn get_page_by_slug(
        &self,
        tenant_id: Uuid,
        slug: &str,
    ) -> Result<Option<PageEntry>, AppError> {
        self.repo
            .get_page_by_slug(tenant_id, slug)
            .await
            .map_err(|e| AppError {
                code: 500,
                message: format!("Failed to fetch page: {}", e),
            })
    }

    /// Get a published page by slug only (for public viewing)
    #[allow(dead_code)]
    pub async fn get_published_page_by_slug(
        &self,
        slug: &str,
    ) -> Result<Option<PageEntry>, AppError> {
        self.repo
            .get_published_page_by_slug(slug)
            .await
            .map_err(|e| AppError {
                code: 500,
                message: format!("Failed to fetch published page: {}", e),
            })
    }

    pub async fn get_page_by_id(
        &self,
        tenant_id: Uuid,
        id: Uuid,
    ) -> Result<Option<PageEntry>, AppError> {
        self.repo
            .get_page_by_id(tenant_id, id)
            .await
            .map_err(|e| AppError {
                code: 500,
                message: format!("Failed to fetch page by ID: {}", e),
            })
    }

    pub async fn list_pages(&self, tenant_id: Uuid) -> Result<Vec<PageEntry>, AppError> {
        self.repo.list_pages(tenant_id).await.map_err(|e| AppError {
            code: 500,
            message: format!("Failed to list pages: {}", e),
        })
    }

    pub async fn delete_page(&self, tenant_id: Uuid, id: Uuid) -> Result<(), AppError> {
        self.repo
            .delete_page(tenant_id, id)
            .await
            .map_err(|e| AppError {
                code: 500,
                message: format!("Failed to delete page: {}", e),
            })
    }

    pub async fn save_page(&self, page: PageEntry) -> Result<(), AppError> {
        self.repo.save_page(page).await.map_err(|e| AppError {
            code: 500,
            message: format!("Failed to save page: {}", e),
        })
    }
}

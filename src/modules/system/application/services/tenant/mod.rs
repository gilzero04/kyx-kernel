use std::sync::Arc;
use crate::modules::system::domain::tenant::{TenantRepository, TenantFilter, PaginatedTenants};
use crate::core::AppError;
use anyhow::Result;

pub struct TenantService {
    repo: Arc<dyn TenantRepository>,
}

impl TenantService {
    pub fn new(repo: Arc<dyn TenantRepository>) -> Self {
        Self { repo }
    }

    pub async fn list_tenants(&self, filter: TenantFilter) -> Result<PaginatedTenants, AppError> {
        self.repo.list(filter).await.map_err(|e| AppError {
            code: 500,
            message: e.to_string(),
        })
    }

    pub async fn update_owner(&self, name: Option<String>, slug: Option<String>) -> Result<(), AppError> {
        self.repo.update_owner(name, slug).await.map_err(|e| AppError {
            code: 500,
            message: e.to_string(),
        })
    }
}

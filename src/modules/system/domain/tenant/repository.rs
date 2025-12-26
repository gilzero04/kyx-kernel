use super::entity::TenantEntry;
use anyhow::Result;
use async_trait::async_trait;
use utoipa::ToSchema;

#[derive(Debug)]
pub struct TenantFilter {
    pub page: i64,
    pub limit: i64,
    pub sort: Option<String>,
    pub search: Option<String>,
}

#[derive(Debug, serde::Serialize, ToSchema)]
pub struct PaginationMetadata {
    pub total: i64,
    pub page: i64,
    pub limit: i64,
    pub total_pages: i64,
}

#[derive(Debug, serde::Serialize, ToSchema)]
pub struct PaginatedTenants {
    pub data: Vec<TenantEntry>,
    pub pagination: PaginationMetadata,
}

#[async_trait]
pub trait TenantRepository: Send + Sync {
    async fn list(&self, filter: TenantFilter) -> Result<PaginatedTenants>;
    async fn update_owner(&self, name: Option<String>, slug: Option<String>) -> Result<()>;
}

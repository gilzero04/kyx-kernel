use super::entity::{UserEntry, TenantMemberCount};
use anyhow::Result;
use async_trait::async_trait;
use uuid::Uuid;
use utoipa::ToSchema;

#[derive(Debug)]
pub struct UserFilter {
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
pub struct PaginatedUsers {
    pub data: Vec<UserEntry>,
    pub pagination: PaginationMetadata,
}

#[async_trait]
pub trait UserRepository: Send + Sync {
    async fn list(&self, filter: UserFilter) -> Result<PaginatedUsers>;
    async fn update(&self, id: Uuid, full_name: Option<String>, is_active: Option<bool>, role_slug: Option<String>, tenant_id: Option<Uuid>) -> Result<bool>; // Returns found/updated
    async fn soft_delete(&self, id: Uuid) -> Result<bool>;
    
    // Checks
    async fn is_superadmin(&self, id: Uuid) -> Result<bool>;
    async fn count_superadmins(&self) -> Result<i64>;
    async fn get_user_tenants(&self, id: Uuid) -> Result<Vec<TenantMemberCount>>;
}

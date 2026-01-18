use super::entity::{TenantMemberCount, UserEntry};
use anyhow::Result;
use async_trait::async_trait;
use utoipa::ToSchema;
use uuid::Uuid;

#[derive(Debug)]
pub struct UserFilter {
    pub page: i64,
    pub limit: i64,
    pub sort: Option<String>,
    pub search: Option<String>,
    pub tenant_id: Option<Uuid>,
    pub actor_tenant_id: Option<Uuid>,
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
    async fn update(
        &self,
        id: Uuid,
        full_name: Option<String>,
        is_active: Option<bool>,
        role_slug: Option<String>,
        tenant_id: Option<Uuid>,
        actor_tenant_id: Option<Uuid>,
    ) -> Result<bool>; // Returns found/updated
    async fn soft_delete(&self, id: Uuid, actor_tenant_id: Option<Uuid>) -> Result<bool>;
    async fn update_password(
        &self,
        id: Uuid,
        hashed_password: String,
        actor_tenant_id: Option<Uuid>,
    ) -> Result<bool>;

    // Checks
    async fn is_superadmin(&self, id: Uuid) -> Result<bool>;
    async fn count_superadmins(&self) -> Result<i64>;
    async fn get_user_tenants(&self, id: Uuid) -> Result<Vec<TenantMemberCount>>;
}

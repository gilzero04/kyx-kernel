use super::entity::{BrandingEntry, TenantEntry};
use anyhow::Result;
use async_trait::async_trait;
use chrono::{DateTime, Utc};
use utoipa::ToSchema;

#[derive(Debug)]
pub struct TenantFilter {
    pub page: i64,
    pub limit: i64,
    pub sort: Option<String>,
    pub search: Option<String>,
    pub actor_tenant_id: Option<sqlx::types::Uuid>,
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

/// Tenant Repository - branding is now managed separately via BrandingRepository
#[async_trait]
pub trait TenantRepository: Send + Sync {
    async fn list(&self, filter: TenantFilter) -> Result<PaginatedTenants>;

    /// Update owner tenant (no branding fields - use BrandingRepository)
    async fn update_owner(
        &self,
        name: Option<String>,
        slug: Option<String>,
        branding_id: Option<sqlx::types::Uuid>,
        contact_email: Option<String>,
        contact_phone: Option<String>,
        website_url: Option<String>,
        social_links: Option<serde_json::Value>,
        address: Option<String>,
        business_type: Option<String>,
        tax_id: Option<String>,
        config: Option<serde_json::Value>,
        custom_domain: Option<String>,
        allow_child_subdomains: Option<bool>,
        domain_verified_at: Option<DateTime<Utc>>,
        verification_token: Option<String>,
    ) -> Result<()>;

    /// Create tenant (no branding fields - use BrandingRepository)
    async fn create(
        &self,
        name: String,
        slug: String,
        type_slug: String,
        plan: Option<String>,
        admin_infos: Option<(String, String, String)>,
        parent_id: Option<sqlx::types::Uuid>,
        branding_id: Option<sqlx::types::Uuid>,
        contact_email: Option<String>,
        contact_phone: Option<String>,
        website_url: Option<String>,
        social_links: Option<serde_json::Value>,
        address: Option<String>,
        business_type: Option<String>,
    ) -> Result<TenantEntry>;

    /// Update tenant (no branding fields - use BrandingRepository)
    async fn update_tenant(
        &self,
        id: sqlx::types::Uuid,
        name: Option<String>,
        slug: Option<String>,
        is_active: Option<bool>,
        branding_id: Option<sqlx::types::Uuid>,
        contact_email: Option<String>,
        contact_phone: Option<String>,
        website_url: Option<String>,
        social_links: Option<serde_json::Value>,
        address: Option<String>,
        business_type: Option<String>,
        config: Option<serde_json::Value>,
        actor_tenant_id: Option<sqlx::types::Uuid>,
        custom_domain: Option<String>,
        allow_child_subdomains: Option<bool>,
        use_parent_subdomain: Option<bool>,
        domain_verified_at: Option<DateTime<Utc>>,
        verification_token: Option<String>,
    ) -> Result<TenantEntry>;

    async fn delete(
        &self,
        id: sqlx::types::Uuid,
        actor_tenant_id: Option<sqlx::types::Uuid>,
    ) -> Result<()>;
    async fn get_owner_id(&self) -> Result<sqlx::types::Uuid>;
    async fn get_by_id(
        &self,
        id: sqlx::types::Uuid,
        actor_tenant_id: Option<sqlx::types::Uuid>,
    ) -> Result<Option<TenantEntry>>;
    async fn get_by_slug(&self, slug: String) -> Result<Option<TenantEntry>>;
    #[allow(dead_code)]
    async fn assign_orphaned_records(&self, tenant_id: sqlx::types::Uuid) -> Result<()>;
}

/// Branding Repository - for CRUD operations on sys_brandings
#[async_trait]
pub trait BrandingRepository: Send + Sync {
    async fn create(&self, branding: BrandingEntry) -> Result<BrandingEntry>;
    async fn update(&self, id: sqlx::types::Uuid, branding: BrandingEntry)
    -> Result<BrandingEntry>;
    async fn get_by_id(&self, id: sqlx::types::Uuid) -> Result<Option<BrandingEntry>>;
    async fn get_by_tenant_id(&self, tenant_id: sqlx::types::Uuid)
    -> Result<Option<BrandingEntry>>;
    async fn delete(&self, id: sqlx::types::Uuid) -> Result<()>;
}

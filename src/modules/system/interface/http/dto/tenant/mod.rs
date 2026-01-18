use chrono::{DateTime, Utc};
use serde::Deserialize;
use utoipa::{IntoParams, ToSchema};

#[derive(Debug, Deserialize, ToSchema, IntoParams)]
pub struct TenantsQuery {
    pub page: Option<i64>,
    pub limit: Option<i64>,
    pub sort: Option<String>,
    pub search: Option<String>,
}

/// Update owner tenant request - branding is now managed separately via branding_id
#[derive(Debug, Deserialize, ToSchema)]
pub struct UpdateOwnerRequest {
    pub name: Option<String>,
    pub slug: Option<String>,
    // Branding reference (use BrandingService to create/update branding)
    pub branding_id: Option<sqlx::types::Uuid>,
    // Contact info
    pub contact_email: Option<String>,
    pub contact_phone: Option<String>,
    pub website_url: Option<String>,
    pub social_links: Option<serde_json::Value>,
    pub address: Option<String>,
    pub business_type: Option<String>,
    pub tax_id: Option<String>,
    // Config and domain
    pub config: Option<serde_json::Value>,
    pub custom_domain: Option<String>,
    pub allow_child_subdomains: Option<bool>,
    pub domain_verified_at: Option<DateTime<Utc>>,
    pub verification_token: Option<String>,
}

/// Create tenant request - branding is managed separately via branding_id
#[derive(Debug, Deserialize, ToSchema)]
pub struct CreateTenantRequest {
    pub name: String,
    pub slug: String,
    pub type_slug: String,
    pub plan: Option<String>,
    pub admin_email: Option<String>,
    pub admin_password: Option<String>,
    pub admin_name: Option<String>,
    pub parent_id: Option<sqlx::types::Uuid>,
    // Branding reference
    pub branding_id: Option<sqlx::types::Uuid>,
    // Contact info
    pub contact_email: Option<String>,
    pub contact_phone: Option<String>,
    pub website_url: Option<String>,
    pub social_links: Option<serde_json::Value>,
    pub address: Option<String>,
    pub business_type: Option<String>,
}

/// Update tenant request - branding is managed separately via branding_id
#[derive(Debug, Deserialize, ToSchema)]
pub struct UpdateTenantRequest {
    pub name: Option<String>,
    pub slug: Option<String>,
    pub is_active: Option<bool>,
    // Branding reference
    pub branding_id: Option<sqlx::types::Uuid>,
    // Contact info
    pub contact_email: Option<String>,
    pub contact_phone: Option<String>,
    pub website_url: Option<String>,
    pub social_links: Option<serde_json::Value>,
    pub address: Option<String>,
    pub business_type: Option<String>,
    // Config and domain
    pub config: Option<serde_json::Value>,
    pub custom_domain: Option<String>,
    pub allow_child_subdomains: Option<bool>,
    pub use_parent_subdomain: Option<bool>,
    pub domain_verified_at: Option<DateTime<Utc>>,
    pub verification_token: Option<String>,
}

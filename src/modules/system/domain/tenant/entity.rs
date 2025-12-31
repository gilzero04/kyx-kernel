use serde::{Deserialize, Serialize};
use chrono::{DateTime, Utc};
use sqlx::FromRow;
use utoipa::ToSchema;

#[derive(Debug, Serialize, Deserialize, FromRow, ToSchema)]
pub struct TenantEntry {
    pub id: sqlx::types::Uuid,
    pub parent_id: Option<sqlx::types::Uuid>,
    pub name: String,
    pub slug: String,
    pub logo_url: Option<String>,
    pub logo_dark_url: Option<String>,
    pub favicon_url: Option<String>,
    pub icon_app_url: Option<String>,
    pub primary_color: Option<String>,
    pub secondary_color: Option<String>,
    pub accent_color: Option<String>,
    pub app_name_override: Option<String>,
    pub contact_email: Option<String>,
    pub contact_phone: Option<String>,
    pub website_url: Option<String>,
    pub social_links: Option<serde_json::Value>,
    pub address: Option<String>,
    pub business_type: Option<String>,
    pub config: Option<serde_json::Value>,
    pub custom_domain: Option<String>,
    pub allow_child_subdomains: Option<bool>,
    pub use_parent_subdomain: Option<bool>,
    pub domain_verified_at: Option<DateTime<Utc>>,
    pub verification_token: Option<String>,
    pub is_active: bool,
    pub created_at: DateTime<Utc>,
    pub member_count: Option<i64>,
}

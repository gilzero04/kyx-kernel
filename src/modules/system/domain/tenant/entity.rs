use serde::{Deserialize, Serialize};
use chrono::{DateTime, Utc};
use sqlx::FromRow;
use utoipa::ToSchema;

/// Tenant entry - branding is now stored in sys_brandings (via branding_id FK)
#[derive(Debug, Serialize, Deserialize, FromRow, ToSchema)]
pub struct TenantEntry {
    pub id: sqlx::types::Uuid,
    pub parent_id: Option<sqlx::types::Uuid>,
    pub name: String,
    pub slug: String,
    // Branding reference (FK to sys_brandings)
    pub branding_id: Option<sqlx::types::Uuid>,
    // Contact info (kept on tenant)
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
    // Status
    pub is_active: bool,
    pub created_at: DateTime<Utc>,
    pub member_count: Option<i64>,
}

/// Branding entry - single source of truth for branding
/// 
/// Contexts:
/// - 'console': Platform/Owner branding for admin console
/// - 'workspace': Tenant branding for workspace (also used by App)
#[derive(Debug, Serialize, Deserialize, FromRow, ToSchema)]
pub struct BrandingEntry {
    pub id: sqlx::types::Uuid,
    pub name: String,
    pub description: Option<String>,
    // Context and ownership
    pub context: Option<String>,  // 'console' or 'workspace'
    pub tenant_id: Option<sqlx::types::Uuid>,
    // Visual assets
    pub logo_light_url: Option<String>,
    pub logo_dark_url: Option<String>,
    pub favicon_url: Option<String>,
    pub icon_app_url: Option<String>,
    pub splash_image_url: Option<String>,
    // Colors
    pub primary_color: Option<String>,
    pub secondary_color: Option<String>,
    pub accent_color: Option<String>,
    // Text
    pub app_name: Option<String>,
    pub splash_text: Option<String>,
    pub splash_subtext: Option<String>,
    pub tagline: Option<String>,
    // Legacy theme references (kept for backward compatibility)
    pub theme_light_id: Option<sqlx::types::Uuid>,
    pub theme_dark_id: Option<sqlx::types::Uuid>,
    // Per-context theme references
    pub theme_console_light_id: Option<sqlx::types::Uuid>,
    pub theme_console_dark_id: Option<sqlx::types::Uuid>,
    pub theme_workspace_light_id: Option<sqlx::types::Uuid>,
    pub theme_workspace_dark_id: Option<sqlx::types::Uuid>,
    pub theme_app_light_id: Option<sqlx::types::Uuid>,
    pub theme_app_dark_id: Option<sqlx::types::Uuid>,
    // Metadata
    pub metadata: Option<serde_json::Value>,
    // Timestamps
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}


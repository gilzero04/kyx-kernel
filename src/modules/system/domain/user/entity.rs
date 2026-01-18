use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use sqlx::FromRow;
use utoipa::ToSchema;

/// User entry - tenant branding is now fetched separately via branding_id
#[derive(Debug, Serialize, Deserialize, FromRow, ToSchema)]
pub struct UserEntry {
    pub id: sqlx::types::Uuid,
    pub email: String,
    pub full_name: Option<String>,
    pub is_active: bool,
    pub created_at: DateTime<Utc>,
    // Joined fields
    pub role: Option<String>,
    pub role_slug: Option<String>,
    pub tenant_name: Option<String>,
    pub tenant_id: Option<sqlx::types::Uuid>,
    // Branding reference (fetch branding separately if needed)
    pub tenant_branding_id: Option<sqlx::types::Uuid>,
    pub permissions: Option<Vec<String>>,
}

#[derive(Debug, Serialize, FromRow, ToSchema)]
pub struct TenantMemberCount {
    pub id: sqlx::types::Uuid,
    pub name: String,
    pub member_count: Option<i64>,
}

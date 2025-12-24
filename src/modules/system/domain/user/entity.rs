use serde::{Deserialize, Serialize};
use sqlx::FromRow;
use chrono::{DateTime, Utc};
use utoipa::ToSchema;

#[derive(Debug, Serialize, Deserialize, FromRow, ToSchema)]
pub struct UserEntry {
    pub id: sqlx::types::Uuid,
    pub email: String,
    pub full_name: Option<String>,
    pub is_active: bool,
    pub created_at: DateTime<Utc>,
    // Joined fields
    pub role: Option<String>,
    pub tenant_name: Option<String>,
}

#[derive(Debug, Serialize, FromRow, ToSchema)]
pub struct TenantMemberCount {
    pub id: sqlx::types::Uuid,
    pub name: String,
    pub member_count: Option<i64>,
}

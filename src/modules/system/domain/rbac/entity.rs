use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use sqlx::FromRow;
use utoipa::ToSchema;
use uuid::Uuid;

#[derive(Debug, Serialize, Deserialize, FromRow, ToSchema)]
pub struct Role {
    pub id: Uuid,
    pub tenant_id: Option<Uuid>,
    pub tenant_name: Option<String>,
    pub code: Option<String>,
    pub slug: String,
    pub name: String,
    pub description: Option<String>,
    pub is_active: bool,
    pub sort_order: i32,
    pub max_members: Option<i32>,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
    pub permission_count: Option<i64>,
    pub member_count: Option<i64>,
}

#[derive(Debug, Serialize, Deserialize, FromRow, ToSchema)]
pub struct Permission {
    pub id: Uuid,
    pub code: Option<String>,
    pub slug: String,
    pub name: String,
    pub description: Option<String>,
    pub is_system: bool,
    pub is_active: bool,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}

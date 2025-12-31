use serde::{Deserialize, Serialize};
use sqlx::FromRow;
use chrono::{DateTime, Utc};
use utoipa::ToSchema;

#[derive(Debug, Serialize, Deserialize, FromRow, ToSchema)]
pub struct MediaFolder {
    pub id: uuid::Uuid,
    pub tenant_id: uuid::Uuid,
    pub parent_id: Option<uuid::Uuid>,
    pub name: String,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}

#[derive(Debug, Serialize, Deserialize, FromRow, ToSchema)]
pub struct MediaAsset {
    pub id: uuid::Uuid,
    pub tenant_id: uuid::Uuid,
    pub folder_id: Option<uuid::Uuid>,
    pub filename: String,
    pub original_name: String,
    pub mime_type: String,
    pub file_size: i64,
    pub url: String,
    pub metadata: Option<serde_json::Value>,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}

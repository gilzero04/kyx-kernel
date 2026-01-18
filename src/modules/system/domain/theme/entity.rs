use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use sqlx::FromRow;
use uuid::Uuid;

#[derive(Debug, Clone, Serialize, Deserialize, FromRow)]
pub struct Theme {
    pub id: Uuid,
    pub slug: Option<String>,
    pub name: String,
    pub description: Option<String>,
    pub config: serde_json::Value,
    pub tenant_id: Option<Uuid>,
    pub is_shared: bool, // Replaces visibility - TRUE = all descendants can see
    pub version: Option<String>,
    pub author: Option<String>,
    pub preview_url: Option<String>,
    pub logo_url: Option<String>,
    pub is_system: bool,
    pub is_active: bool,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}

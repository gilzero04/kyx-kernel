use serde::{Deserialize, Serialize};
use sqlx::FromRow;
use uuid::Uuid;
use chrono::{DateTime, Utc};

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize, sqlx::Type)]
#[sqlx(type_name = "theme_visibility", rename_all = "lowercase")]
pub enum ThemeVisibility {
    Public,
    Private,
    Restricted,
}

#[derive(Debug, Clone, Serialize, Deserialize, FromRow)]
pub struct Theme {
    pub id: Uuid,
    pub name: String,
    pub description: Option<String>,
    pub config: serde_json::Value,
    pub tenant_id: Option<Uuid>,
    pub visibility: ThemeVisibility,
    pub version: Option<String>,
    pub author: Option<String>,
    pub preview_url: Option<String>,
    pub logo_url: Option<String>,
    pub is_active: bool,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}


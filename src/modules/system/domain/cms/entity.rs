use serde::{Deserialize, Serialize};
use uuid::Uuid;
use chrono::{DateTime, Utc};

#[derive(Debug, Serialize, Deserialize)]
pub struct PageEntry {
    pub id: Uuid,
    pub tenant_id: Uuid,
    pub slug: String,
    pub title: String,
    pub content: serde_json::Value,
    pub is_published: bool,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}

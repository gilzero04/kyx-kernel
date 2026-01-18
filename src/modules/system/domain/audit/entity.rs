use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use serde_json::Value;
use utoipa::ToSchema;

#[derive(Debug, Serialize, Deserialize, sqlx::FromRow, ToSchema)]
pub struct AuditLogEntry {
    pub id: sqlx::types::Uuid,
    pub timestamp: DateTime<Utc>,
    pub actor: String,
    pub action: String,
    pub target: Option<String>,
    pub status: String,
    pub metadata: Option<Value>,
}

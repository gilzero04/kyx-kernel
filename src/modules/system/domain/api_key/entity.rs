use serde::{Deserialize, Serialize};
use serde_json::Value;
use sqlx::FromRow;
use utoipa::ToSchema;
use uuid::Uuid;

#[derive(Debug, Serialize, Deserialize, FromRow, ToSchema)]
pub struct ApiKey {
    pub id: Uuid,
    pub tenant_id: Uuid,
    pub prefix: String,
    pub name: Option<String>,
    pub key_type: String,
    pub allowed_origins: Option<Value>,
    pub is_active: bool,
    #[sqlx(default)]
    pub key_hash: String, // Ensure we can load hash for verification
}

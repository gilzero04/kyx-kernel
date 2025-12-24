use serde::{Deserialize, Serialize};
use sqlx::FromRow;
use utoipa::ToSchema;

#[derive(Debug, Serialize, Deserialize, FromRow, ToSchema)]
pub struct CorsOrigin {
    pub id: i32,
    pub origin: String,
    pub is_active: Option<bool>,
    pub description: Option<String>,
}

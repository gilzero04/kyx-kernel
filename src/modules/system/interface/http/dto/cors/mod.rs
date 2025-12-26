use serde::Deserialize;
use utoipa::ToSchema;

#[derive(Deserialize, ToSchema)]
pub struct AddCorsRequest {
    pub origin: String,
    pub description: Option<String>,
}

#[derive(Deserialize, ToSchema)]
pub struct UpdateCorsRequest {
    pub is_active: Option<bool>,
    pub description: Option<String>,
}

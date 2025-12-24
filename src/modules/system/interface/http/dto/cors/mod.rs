use serde::Deserialize;
use utoipa::ToSchema;

#[derive(Deserialize, ToSchema)]
pub struct AddCorsRequest {
    pub origin: String,
    pub description: Option<String>,
}

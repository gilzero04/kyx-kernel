use serde::Deserialize;
use utoipa::ToSchema;

#[derive(Debug, Deserialize, ToSchema)]
pub struct CreateApiKeyRequest {
    pub tenant_id: uuid::Uuid,
    pub name: Option<String>,
    pub key_type: String, // server | client
}

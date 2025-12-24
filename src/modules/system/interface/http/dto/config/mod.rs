use serde::Deserialize;
use serde_json::Value;
use utoipa::ToSchema;

#[derive(Deserialize, ToSchema)]
pub struct ConfigUpdate {
    pub key: String,
    pub value: Value,
}

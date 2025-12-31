use serde::{Deserialize, Serialize};
use utoipa::ToSchema;

#[derive(Debug, Serialize, ToSchema)]
pub struct TranslationsResponse {
    pub locale: String,
    pub translations: std::collections::HashMap<String, String>,
}

#[derive(Debug, Deserialize, ToSchema)]
pub struct CreateI18nKeyRequest {
    pub key: String,
    pub default_message: String, 
}

#[derive(Debug, Deserialize, ToSchema)]
pub struct UpdateTranslationRequest {
    #[allow(dead_code)]
    pub message: String,
}

#[derive(Debug, Deserialize, ToSchema)]
pub struct CreateLocaleRequest {
    pub code: String,
    pub name: String,
}

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
    /// Optional tenant ID for tenant-specific translations
    pub tenant_id: Option<String>,
    /// Optional context: 'console' or 'workspace' (App uses workspace)
    pub context: Option<String>,
}

#[derive(Debug, Deserialize, ToSchema)]
pub struct UpdateTranslationRequest {
    pub locale: String,
    pub key: String,
    pub message: String,
    /// Optional tenant ID for tenant-specific translations
    pub tenant_id: Option<String>,
    /// Optional context: 'console' or 'workspace' (App uses workspace)
    pub context: Option<String>,
}

#[derive(Debug, Deserialize, ToSchema)]
pub struct CreateLocaleRequest {
    pub code: String,
    pub name: String,
}


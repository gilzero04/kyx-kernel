use serde::{Deserialize, Serialize};
use uuid::Uuid;
use crate::modules::system::domain::theme::ThemeVisibility;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CreateThemeDto {
    pub name: String,
    pub description: Option<String>,
    pub config: serde_json::Value,
    pub visibility: ThemeVisibility,
    pub tenant_id: Option<Uuid>,
    pub author: Option<String>,
    pub preview_url: Option<String>,
    pub logo_url: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct UpdateThemeDto {
    pub name: Option<String>,
    pub description: Option<String>,
    pub config: Option<serde_json::Value>,
    pub visibility: Option<ThemeVisibility>,
    pub is_active: Option<bool>,
    pub author: Option<String>,
    pub preview_url: Option<String>,
    pub logo_url: Option<String>,
}

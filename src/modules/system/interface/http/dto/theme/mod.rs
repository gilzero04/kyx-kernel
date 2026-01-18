use serde::{Deserialize, Serialize};
use uuid::Uuid;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CreateThemeDto {
    /// Optional ID from manifest.json (UUID format)
    /// If not provided, database will auto-generate
    pub id: Option<Uuid>,
    /// Human-readable slug (e.g., "kyx-dark", "pixco-ocean")
    pub slug: Option<String>,
    pub name: String,
    pub description: Option<String>,
    pub config: serde_json::Value,
    pub is_shared: bool, // Replaces visibility - TRUE = broadcast to descendants
    pub tenant_id: Option<Uuid>,
    pub author: Option<String>,
    pub preview_url: Option<String>,
    pub logo_url: Option<String>,
    /// Semantic version (e.g., "1.0.0")
    pub version: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct UpdateThemeDto {
    /// Human-readable slug
    pub slug: Option<String>,
    pub name: Option<String>,
    pub description: Option<String>,
    pub config: Option<serde_json::Value>,
    pub is_shared: Option<bool>, // Replaces visibility
    pub is_active: Option<bool>,
    pub author: Option<String>,
    pub preview_url: Option<String>,
    pub logo_url: Option<String>,
    /// Semantic version (e.g., "1.0.0")
    pub version: Option<String>,
}

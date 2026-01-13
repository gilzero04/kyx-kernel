use serde::{Serialize, Deserialize};
use chrono::{DateTime, Utc};

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct Locale {
    pub code: String,
    pub name: String,
    pub is_active: bool,
    pub is_default: bool,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct Translation {
    pub id: i32,
    pub locale: String,
    pub key: String,
    pub message: String,
    pub is_auto_generated: bool,
    // Context separation fields
    pub tenant_id: Option<sqlx::types::Uuid>,
    pub context: Option<String>,  // 'console' or 'workspace' (App uses workspace)
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}

/// i18n context type for clarity
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum I18nContext {
    /// Global platform translations (default)
    Global,
    /// Console-specific translations
    Console,
    /// Workspace and App translations (shared)
    Workspace,
}

impl I18nContext {
    pub fn as_str(&self) -> Option<&'static str> {
        match self {
            I18nContext::Global => None,
            I18nContext::Console => Some("console"),
            I18nContext::Workspace => Some("workspace"),
        }
    }
    
    pub fn from_str(s: Option<&str>) -> Self {
        match s {
            Some("console") => I18nContext::Console,
            Some("workspace") => I18nContext::Workspace,
            _ => I18nContext::Global,
        }
    }
}


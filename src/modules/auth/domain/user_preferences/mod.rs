use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use sqlx::FromRow;
use uuid::Uuid;

/// User preferences entity for per-user settings like theme
#[derive(Debug, Clone, Serialize, Deserialize, FromRow)]
pub struct UserPreferences {
    pub id: Uuid,
    pub user_id: Uuid,
    pub preferred_theme_light_id: Option<String>,
    pub preferred_theme_dark_id: Option<String>,
    pub theme_mode: Option<String>, // 'light', 'dark', 'system'
    pub locale: Option<String>,
    pub timezone: Option<String>,
    pub notifications_enabled: Option<bool>,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}

/// DTO for updating user preferences
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct UpdateUserPreferencesDto {
    pub preferred_theme_light_id: Option<String>,
    pub preferred_theme_dark_id: Option<String>,
    pub theme_mode: Option<String>,
    pub locale: Option<String>,
    pub timezone: Option<String>,
    pub notifications_enabled: Option<bool>,
}

impl UserPreferences {
    pub fn new(user_id: Uuid) -> Self {
        Self {
            id: Uuid::new_v4(),
            user_id,
            preferred_theme_light_id: None,
            preferred_theme_dark_id: None,
            theme_mode: Some("system".to_string()),
            locale: None,
            timezone: None,
            notifications_enabled: Some(true),
            created_at: Utc::now(),
            updated_at: Utc::now(),
        }
    }
}

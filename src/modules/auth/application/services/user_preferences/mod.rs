use std::sync::Arc;
use uuid::Uuid;

use crate::core::infrastructure::database::Database;
use crate::modules::auth::domain::user_preferences::{UserPreferences, UpdateUserPreferencesDto};
use crate::modules::auth::infrastructure::repositories::user_preferences::{
    PostgresUserPreferencesRepository, UserPreferencesRepository
};

/// Service for managing user preferences (theme, locale, etc.)
pub struct UserPreferencesService {
    repo: PostgresUserPreferencesRepository,
}

#[allow(dead_code)]
impl UserPreferencesService {
    pub fn new(db: Arc<Database>) -> Self {
        Self {
            repo: PostgresUserPreferencesRepository::new(db),
        }
    }

    /// Get user preferences, returns None if not set
    pub async fn get_preferences(&self, user_id: Uuid) -> Result<Option<UserPreferences>, String> {
        self.repo
            .get_by_user_id(user_id)
            .await
            .map_err(|e| format!("Failed to get preferences: {}", e))
    }

    /// Get or create default preferences for a user
    pub async fn get_or_create_preferences(&self, user_id: Uuid) -> Result<UserPreferences, String> {
        match self.repo.get_by_user_id(user_id).await {
            Ok(Some(prefs)) => Ok(prefs),
            Ok(None) => {
                let new_prefs = UserPreferences::new(user_id);
                self.repo
                    .create(&new_prefs)
                    .await
                    .map_err(|e| format!("Failed to create preferences: {}", e))
            }
            Err(e) => Err(format!("Failed to get preferences: {}", e)),
        }
    }

    /// Update user preferences (upsert)
    pub async fn update_preferences(
        &self,
        user_id: Uuid,
        dto: UpdateUserPreferencesDto,
    ) -> Result<UserPreferences, String> {
        self.repo
            .upsert(user_id, &dto)
            .await
            .map_err(|e| format!("Failed to update preferences: {}", e))
    }

    /// Set user theme preferences
    pub async fn set_theme(
        &self,
        user_id: Uuid,
        light_theme: Option<String>,
        dark_theme: Option<String>,
    ) -> Result<UserPreferences, String> {
        let dto = UpdateUserPreferencesDto {
            preferred_theme_light_id: light_theme,
            preferred_theme_dark_id: dark_theme,
            theme_mode: None,
            locale: None,
            timezone: None,
            notifications_enabled: None,
        };
        self.update_preferences(user_id, dto).await
    }

    /// Set user theme mode (light/dark/system)
    pub async fn set_theme_mode(
        &self,
        user_id: Uuid,
        mode: String,
    ) -> Result<UserPreferences, String> {
        let dto = UpdateUserPreferencesDto {
            preferred_theme_light_id: None,
            preferred_theme_dark_id: None,
            theme_mode: Some(mode),
            locale: None,
            timezone: None,
            notifications_enabled: None,
        };
        self.update_preferences(user_id, dto).await
    }
}

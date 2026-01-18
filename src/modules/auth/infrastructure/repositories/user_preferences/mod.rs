use std::sync::Arc;
use uuid::Uuid;

use crate::core::infrastructure::database::Database;
use crate::modules::auth::domain::user_preferences::{UpdateUserPreferencesDto, UserPreferences};

#[allow(dead_code)]
pub trait UserPreferencesRepository: Send + Sync {
    fn get_by_user_id(
        &self,
        user_id: Uuid,
    ) -> impl std::future::Future<Output = Result<Option<UserPreferences>, sqlx::Error>> + Send;
    fn create(
        &self,
        prefs: &UserPreferences,
    ) -> impl std::future::Future<Output = Result<UserPreferences, sqlx::Error>> + Send;
    fn update(
        &self,
        user_id: Uuid,
        dto: &UpdateUserPreferencesDto,
    ) -> impl std::future::Future<Output = Result<UserPreferences, sqlx::Error>> + Send;
    fn upsert(
        &self,
        user_id: Uuid,
        dto: &UpdateUserPreferencesDto,
    ) -> impl std::future::Future<Output = Result<UserPreferences, sqlx::Error>> + Send;
}

pub struct PostgresUserPreferencesRepository {
    db: Arc<Database>,
}

impl PostgresUserPreferencesRepository {
    pub fn new(db: Arc<Database>) -> Self {
        Self { db }
    }
}

impl UserPreferencesRepository for PostgresUserPreferencesRepository {
    async fn get_by_user_id(&self, user_id: Uuid) -> Result<Option<UserPreferences>, sqlx::Error> {
        sqlx::query_as::<_, UserPreferences>(
            r#"
            SELECT id, user_id, preferred_theme_light_id, preferred_theme_dark_id, 
                   theme_mode, locale, timezone, notifications_enabled, created_at, updated_at
            FROM user_preferences
            WHERE user_id = $1
            "#,
        )
        .bind(user_id)
        .fetch_optional(&self.db.pool)
        .await
    }

    async fn create(&self, prefs: &UserPreferences) -> Result<UserPreferences, sqlx::Error> {
        sqlx::query_as::<_, UserPreferences>(
            r#"
            INSERT INTO user_preferences (
                id, user_id, preferred_theme_light_id, preferred_theme_dark_id,
                theme_mode, locale, timezone, notifications_enabled, created_at, updated_at
            ) VALUES ($1, $2, $3, $4, $5, $6, $7, $8, $9, $10)
            RETURNING id, user_id, preferred_theme_light_id, preferred_theme_dark_id,
                      theme_mode, locale, timezone, notifications_enabled, created_at, updated_at
            "#,
        )
        .bind(prefs.id)
        .bind(prefs.user_id)
        .bind(&prefs.preferred_theme_light_id)
        .bind(&prefs.preferred_theme_dark_id)
        .bind(&prefs.theme_mode)
        .bind(&prefs.locale)
        .bind(&prefs.timezone)
        .bind(prefs.notifications_enabled)
        .bind(prefs.created_at)
        .bind(prefs.updated_at)
        .fetch_one(&self.db.pool)
        .await
    }

    async fn update(
        &self,
        user_id: Uuid,
        dto: &UpdateUserPreferencesDto,
    ) -> Result<UserPreferences, sqlx::Error> {
        sqlx::query_as::<_, UserPreferences>(
            r#"
            UPDATE user_preferences SET
                preferred_theme_light_id = COALESCE($2, preferred_theme_light_id),
                preferred_theme_dark_id = COALESCE($3, preferred_theme_dark_id),
                theme_mode = COALESCE($4, theme_mode),
                locale = COALESCE($5, locale),
                timezone = COALESCE($6, timezone),
                notifications_enabled = COALESCE($7, notifications_enabled),
                updated_at = NOW()
            WHERE user_id = $1
            RETURNING id, user_id, preferred_theme_light_id, preferred_theme_dark_id,
                      theme_mode, locale, timezone, notifications_enabled, created_at, updated_at
            "#,
        )
        .bind(user_id)
        .bind(&dto.preferred_theme_light_id)
        .bind(&dto.preferred_theme_dark_id)
        .bind(&dto.theme_mode)
        .bind(&dto.locale)
        .bind(&dto.timezone)
        .bind(dto.notifications_enabled)
        .fetch_one(&self.db.pool)
        .await
    }

    async fn upsert(
        &self,
        user_id: Uuid,
        dto: &UpdateUserPreferencesDto,
    ) -> Result<UserPreferences, sqlx::Error> {
        sqlx::query_as::<_, UserPreferences>(
            r#"
            INSERT INTO user_preferences (
                id, user_id, preferred_theme_light_id, preferred_theme_dark_id,
                theme_mode, locale, timezone, notifications_enabled, created_at, updated_at
            ) VALUES (gen_random_uuid(), $1, $2, $3, $4, $5, $6, $7, NOW(), NOW())
            ON CONFLICT (user_id) DO UPDATE SET
                preferred_theme_light_id = COALESCE(EXCLUDED.preferred_theme_light_id, user_preferences.preferred_theme_light_id),
                preferred_theme_dark_id = COALESCE(EXCLUDED.preferred_theme_dark_id, user_preferences.preferred_theme_dark_id),
                theme_mode = COALESCE(EXCLUDED.theme_mode, user_preferences.theme_mode),
                locale = COALESCE(EXCLUDED.locale, user_preferences.locale),
                timezone = COALESCE(EXCLUDED.timezone, user_preferences.timezone),
                notifications_enabled = COALESCE(EXCLUDED.notifications_enabled, user_preferences.notifications_enabled),
                updated_at = NOW()
            RETURNING id, user_id, preferred_theme_light_id, preferred_theme_dark_id,
                      theme_mode, locale, timezone, notifications_enabled, created_at, updated_at
            "#
        )
        .bind(user_id)
        .bind(&dto.preferred_theme_light_id)
        .bind(&dto.preferred_theme_dark_id)
        .bind(&dto.theme_mode)
        .bind(&dto.locale)
        .bind(&dto.timezone)
        .bind(dto.notifications_enabled)
        .fetch_one(&self.db.pool)
        .await
    }
}

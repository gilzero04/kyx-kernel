use async_trait::async_trait;
use anyhow::Result;
use crate::modules::system::domain::i18n::{I18nRepository, Locale, Translation};
use std::sync::Arc;

#[derive(Clone)]
pub struct PostgresI18nRepositoryImpl {
    db: Arc<crate::core::infrastructure::database::Database>,
}

impl PostgresI18nRepositoryImpl {
    pub fn new(db: Arc<crate::core::infrastructure::database::Database>) -> Self {
        Self { db }
    }
}

#[async_trait]
impl I18nRepository for PostgresI18nRepositoryImpl {
    async fn list_locales(&self) -> Result<Vec<Locale>> {
        let locales = sqlx::query_as!(
            Locale,
            r#"SELECT code, name, is_active as "is_active!", is_default as "is_default!", created_at as "created_at!", updated_at as "updated_at!" 
               FROM sys_i18n_locales WHERE is_active = TRUE ORDER BY is_default DESC, code ASC"#
        )
        .fetch_all(&self.db.pool)
        .await?;
        Ok(locales)
    }

    async fn get_translations(&self, locale: &str) -> Result<Vec<Translation>> {
        let rows = sqlx::query_as!(
            Translation,
            r#"SELECT id, locale, key, message, is_auto_generated as "is_auto_generated!", created_at as "created_at!", updated_at as "updated_at!" 
               FROM sys_i18n_translations WHERE locale = $1"#,
            locale
        )
        .fetch_all(&self.db.pool)
        .await?;
        Ok(rows)
    }

    async fn create_key(&self, locale: &str, key: &str, message: &str) -> Result<()> {
        sqlx::query(
            "INSERT INTO sys_i18n_translations (locale, key, message) VALUES ($1, $2, $3) ON CONFLICT (locale, key) DO NOTHING"
        )
        .bind(locale)
        .bind(key)
        .bind(message)
        .execute(&self.db.pool)
        .await?;
        Ok(())
    }

    async fn update_translation(&self, locale: &str, key: &str, message: &str) -> Result<()> {
         sqlx::query(
            "INSERT INTO sys_i18n_translations (locale, key, message) VALUES ($1, $2, $3) 
             ON CONFLICT (locale, key) DO UPDATE SET message = $3, updated_at = NOW()"
        )
        .bind(locale)
        .bind(key)
        .bind(message)
        .execute(&self.db.pool)
        .await?;
        Ok(())
    }

    async fn create_locale(&self, code: &str, name: &str) -> Result<()> {
        sqlx::query(
            "INSERT INTO sys_i18n_locales (code, name) VALUES ($1, $2) ON CONFLICT (code) DO NOTHING"
        )
        .bind(code)
        .bind(name)
        .execute(&self.db.pool)
        .await?;
        Ok(())
    }

    async fn delete_key(&self, key: &str) -> Result<()> {
        sqlx::query("DELETE FROM sys_i18n_translations WHERE key = $1")
        .bind(key)
        .execute(&self.db.pool)
        .await?;
        Ok(())
    }

    async fn delete_locale(&self, code: &str) -> Result<()> {
        // Delete translations first (though FK cascade should handle it ideally)
        sqlx::query("DELETE FROM sys_i18n_translations WHERE locale = $1")
        .bind(code)
        .execute(&self.db.pool)
        .await?;

        // Delete locale
        sqlx::query("DELETE FROM sys_i18n_locales WHERE code = $1")
        .bind(code)
        .execute(&self.db.pool)
        .await?;
        Ok(())
    }
}

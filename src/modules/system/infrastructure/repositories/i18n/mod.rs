use async_trait::async_trait;
use anyhow::Result;
use crate::modules::system::domain::i18n::{I18nRepository, Locale, Translation};
use std::sync::Arc;
use sqlx::types::Uuid;
use sqlx::Row;

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

    async fn get_translations(&self, locale: &str, tenant_id: Option<Uuid>, context: Option<&str>) -> Result<Vec<Translation>> {
        // Use a query that returns tenant-specific translations with fallback to global
        let rows = sqlx::query(
            r#"
            SELECT 
                COALESCE(t.id, g.id) as id,
                COALESCE(t.locale, g.locale) as locale,
                COALESCE(t.key, g.key) as key,
                COALESCE(t.message, g.message) as message,
                COALESCE(t.is_auto_generated, g.is_auto_generated) as is_auto_generated,
                t.tenant_id,
                t.context,
                COALESCE(t.created_at, g.created_at) as created_at,
                COALESCE(t.updated_at, g.updated_at) as updated_at
            FROM sys_i18n_translations g
            LEFT JOIN sys_i18n_translations t ON 
                g.locale = t.locale 
                AND g.key = t.key 
                AND t.tenant_id = $2
                AND (t.context = $3 OR ($3 IS NULL AND t.context IS NULL))
            WHERE g.locale = $1 
                AND g.tenant_id IS NULL 
                AND g.context IS NULL
            ORDER BY g.key
            "#
        )
        .bind(locale)
        .bind(tenant_id)
        .bind(context)
        .fetch_all(&self.db.pool)
        .await?;

        let translations: Vec<Translation> = rows.iter().map(|row| Translation {
            id: row.get("id"),
            locale: row.get("locale"),
            key: row.get("key"),
            message: row.get("message"),
            is_auto_generated: row.get("is_auto_generated"),
            tenant_id: row.get("tenant_id"),
            context: row.get("context"),
            created_at: row.get("created_at"),
            updated_at: row.get("updated_at"),
        }).collect();

        Ok(translations)
    }

    async fn create_key(&self, locale: &str, key: &str, message: &str, tenant_id: Option<Uuid>, context: Option<&str>) -> Result<()> {
        sqlx::query(
            r#"INSERT INTO sys_i18n_translations (locale, key, message, tenant_id, context) 
               VALUES ($1, $2, $3, $4, $5) 
               ON CONFLICT DO NOTHING"#
        )
        .bind(locale)
        .bind(key)
        .bind(message)
        .bind(tenant_id)
        .bind(context)
        .execute(&self.db.pool)
        .await?;
        Ok(())
    }

    async fn update_translation(&self, locale: &str, key: &str, message: &str, tenant_id: Option<Uuid>, context: Option<&str>) -> Result<()> {
        // For tenant-specific: upsert with tenant_id and context
        // For global: update existing or insert
        if tenant_id.is_some() {
            sqlx::query(
                r#"INSERT INTO sys_i18n_translations (locale, key, message, tenant_id, context) 
                   VALUES ($1, $2, $3, $4, $5) 
                   ON CONFLICT (locale, key, COALESCE(tenant_id, '00000000-0000-0000-0000-000000000000'::uuid), COALESCE(context, 'global')) 
                   DO UPDATE SET message = $3, updated_at = NOW()"#
            )
            .bind(locale)
            .bind(key)
            .bind(message)
            .bind(tenant_id)
            .bind(context)
            .execute(&self.db.pool)
            .await?;
        } else {
            // Global translation update
            sqlx::query(
                r#"UPDATE sys_i18n_translations 
                   SET message = $3, updated_at = NOW() 
                   WHERE locale = $1 AND key = $2 AND tenant_id IS NULL AND context IS NULL"#
            )
            .bind(locale)
            .bind(key)
            .bind(message)
            .execute(&self.db.pool)
            .await?;
        }
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

    async fn delete_key(&self, key: &str, tenant_id: Option<Uuid>, context: Option<&str>) -> Result<()> {
        if tenant_id.is_some() {
            // Delete only tenant-specific translation
            sqlx::query(
                r#"DELETE FROM sys_i18n_translations 
                   WHERE key = $1 AND tenant_id = $2 AND (context = $3 OR ($3 IS NULL AND context IS NULL))"#
            )
            .bind(key)
            .bind(tenant_id)
            .bind(context)
            .execute(&self.db.pool)
            .await?;
        } else {
            // Delete global translation
            sqlx::query("DELETE FROM sys_i18n_translations WHERE key = $1 AND tenant_id IS NULL")
                .bind(key)
                .execute(&self.db.pool)
                .await?;
        }
        Ok(())
    }

    async fn delete_locale(&self, code: &str) -> Result<()> {
        // Deactivate locale instead of hard delete
        sqlx::query("UPDATE sys_i18n_locales SET is_active = FALSE, updated_at = NOW() WHERE code = $1")
            .bind(code)
            .execute(&self.db.pool)
            .await?;
        Ok(())
    }
}


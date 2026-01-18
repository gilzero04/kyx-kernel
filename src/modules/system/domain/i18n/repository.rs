use super::entity::{Locale, Translation};
use anyhow::Result;
use async_trait::async_trait;
use sqlx::types::Uuid;

#[async_trait]
pub trait I18nRepository: Send + Sync {
    async fn list_locales(&self) -> Result<Vec<Locale>>;

    /// Get translations for a locale, optionally filtered by tenant and context
    /// Falls back to global translations if no tenant-specific found
    async fn get_translations(
        &self,
        locale: &str,
        tenant_id: Option<Uuid>,
        context: Option<&str>,
    ) -> Result<Vec<Translation>>;

    /// Create a translation key, optionally for a specific tenant/context
    async fn create_key(
        &self,
        locale: &str,
        key: &str,
        message: &str,
        tenant_id: Option<Uuid>,
        context: Option<&str>,
    ) -> Result<()>;

    /// Update a translation, optionally for a specific tenant/context
    async fn update_translation(
        &self,
        locale: &str,
        key: &str,
        message: &str,
        tenant_id: Option<Uuid>,
        context: Option<&str>,
    ) -> Result<()>;

    async fn create_locale(&self, code: &str, name: &str) -> Result<()>;

    /// Delete a key, optionally only for a specific tenant/context
    async fn delete_key(
        &self,
        key: &str,
        tenant_id: Option<Uuid>,
        context: Option<&str>,
    ) -> Result<()>;

    async fn delete_locale(&self, code: &str) -> Result<()>;
}

use async_trait::async_trait;
use anyhow::Result;
use super::entity::{Locale, Translation};

#[async_trait]
pub trait I18nRepository: Send + Sync {
    async fn list_locales(&self) -> Result<Vec<Locale>>;
    async fn get_translations(&self, locale: &str) -> Result<Vec<Translation>>;
    async fn create_key(&self, locale: &str, key: &str, message: &str) -> Result<()>;
    async fn update_translation(&self, locale: &str, key: &str, message: &str) -> Result<()>;
    async fn create_locale(&self, code: &str, name: &str) -> Result<()>;
    async fn delete_key(&self, key: &str) -> Result<()>;
    async fn delete_locale(&self, code: &str) -> Result<()>;
}

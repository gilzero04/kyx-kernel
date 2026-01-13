use std::sync::Arc;
use anyhow::Result;
use crate::modules::system::domain::i18n::{I18nRepository, Locale};
use std::collections::HashMap;
use crate::core::infrastructure::audit::AuditService;
use crate::core::infrastructure::ai_service::AIService;
use sqlx::types::Uuid;

pub struct I18nService {
    repo: Arc<dyn I18nRepository>,
    #[allow(dead_code)]
    audit: Arc<AuditService>,
    ai: Arc<AIService>,
}

impl I18nService {
    pub fn new(repo: Arc<dyn I18nRepository>, audit: Arc<AuditService>, ai: Arc<AIService>) -> Self {
        Self { repo, audit, ai }
    }

    pub async fn list_locales(&self) -> Result<Vec<Locale>> {
        self.repo.list_locales().await
    }

    /// Get translations map for a locale, optionally filtered by tenant and context
    pub async fn get_translations_map(&self, locale: &str, tenant_id: Option<Uuid>, context: Option<&str>) -> Result<HashMap<String, String>> {
        let translations = self.repo.get_translations(locale, tenant_id, context).await?;
        let mut map = HashMap::new();
        for t in translations {
            map.insert(t.key, t.message);
        }
        Ok(map)
    }

    /// Create a translation key, optionally for a specific tenant/context
    pub async fn create_key(&self, key: &str, default_message: &str, tenant_id: Option<Uuid>, context: Option<&str>) -> Result<()> {
        let locales = self.repo.list_locales().await?;
        for locale in locales {
            self.repo.create_key(&locale.code, key, default_message, tenant_id, context).await?;
        }
        Ok(())
    }

    /// Update a translation, optionally for a specific tenant/context
    pub async fn update_translation(&self, locale: &str, key: &str, message: &str, tenant_id: Option<Uuid>, context: Option<&str>) -> Result<()> {
        self.repo.update_translation(locale, key, message, tenant_id, context).await
    }

    pub async fn create_locale(&self, code: &str, name: &str) -> Result<()> {
        self.repo.create_locale(code, name).await?;

        // Auto-populate from default locale (global translations only)
        let locales = self.repo.list_locales().await?;
        let default_locale = locales.iter().find(|l| l.is_default).or(locales.first());

        if let Some(source) = default_locale {
            let translations = self.repo.get_translations(&source.code, None, None).await?;
            
            // Process in batches of 50 to avoid API limits (Context Window)
            for chunk in translations.chunks(50) {
                let keys: Vec<String> = chunk.iter().map(|t| t.key.clone()).collect();
                let texts: Vec<String> = chunk.iter().map(|t| t.message.clone()).collect();

                // Translate batch
                let translated_texts = self.ai.translate_batch(texts.clone(), code).await;

                // Insert global translations for new locale
                for (i, key) in keys.iter().enumerate() {
                    let msg = translated_texts.get(i).unwrap_or(&texts[i]);
                    let _ = self.repo.create_key(code, key, msg, None, None).await;
                }
            }
        }
        Ok(())
    }

    /// Delete a key, optionally only for a specific tenant/context
    pub async fn delete_key(&self, key: &str, tenant_id: Option<Uuid>, context: Option<&str>) -> Result<()> {
        self.repo.delete_key(key, tenant_id, context).await
    }

    pub async fn delete_locale(&self, code: &str) -> Result<()> {
        self.repo.delete_locale(code).await
    }

    /// List all translations (global only for admin management)
    pub async fn list_all_translations(&self) -> Result<Vec<serde_json::Value>> {
        let locales = self.repo.list_locales().await?;
        let mut result = Vec::new();
        for locale in locales {
            let translations = self.repo.get_translations(&locale.code, None, None).await?;
            result.push(serde_json::json!({
                "locale": locale.code,
                "name": locale.name,
                "count": translations.len(),
                "translations": translations.into_iter().map(|t| serde_json::json!({
                    "key": t.key,
                    "message": t.message
                })).collect::<Vec<_>>()
            }));
        }
        Ok(result)
    }
}


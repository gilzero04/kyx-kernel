use std::sync::Arc;
use anyhow::Result;
use crate::modules::system::domain::i18n::{I18nRepository, Locale};
use std::collections::HashMap;
use crate::core::infrastructure::audit::AuditService;
use crate::core::infrastructure::ai_service::AIService;

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

    pub async fn get_translations_map(&self, locale: &str) -> Result<HashMap<String, String>> {
        let translations = self.repo.get_translations(locale).await?;
        let mut map = HashMap::new();
        for t in translations {
            map.insert(t.key, t.message);
        }
        Ok(map)
    }

    pub async fn create_key(&self, key: &str, default_message: &str) -> Result<()> {
        let locales = self.repo.list_locales().await?;
        for locale in locales {
            self.repo.create_key(&locale.code, key, default_message).await?;
        }
        Ok(())
    }

    pub async fn update_translation(&self, locale: &str, key: &str, message: &str) -> Result<()> {
        self.repo.update_translation(locale, key, message).await
    }

    pub async fn create_locale(&self, code: &str, name: &str) -> Result<()> {
        self.repo.create_locale(code, name).await?;

        // Auto-populate from default locale
        let locales = self.repo.list_locales().await?;
        let default_locale = locales.iter().find(|l| l.is_default).or(locales.first());

        if let Some(source) = default_locale {
            let translations = self.repo.get_translations(&source.code).await?;
            
            // Process in batches of 50 to avoid API limits (Context Window)
            for chunk in translations.chunks(50) {
                let keys: Vec<String> = chunk.iter().map(|t| t.key.clone()).collect();
                let texts: Vec<String> = chunk.iter().map(|t| t.message.clone()).collect();

                // Translate batch
                let translated_texts = self.ai.translate_batch(texts.clone(), code).await;

                // Insert
                for (i, key) in keys.iter().enumerate() {
                    let msg = translated_texts.get(i).unwrap_or(&texts[i]);
                    let _ = self.repo.create_key(code, key, msg).await;
                }
            }
        }
        Ok(())
    }

    pub async fn delete_key(&self, key: &str) -> Result<()> {
        self.repo.delete_key(key).await
    }

    pub async fn delete_locale(&self, code: &str) -> Result<()> {
        self.repo.delete_locale(code).await
    }
}

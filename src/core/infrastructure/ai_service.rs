use std::sync::Arc;
use ntex::http::client::Client;
// use ntex::http::header; // Unused
use serde_json::{json, Value};
use log::{error, info};
use crate::core::infrastructure::config_service::ConfigService;

#[derive(Clone)]
pub struct AIService {
    config: Arc<ConfigService>,
}

impl AIService {
    pub fn new(config: Arc<ConfigService>) -> Self {
        Self {
            config,
        }
    }

    /// Translate a batch of texts to the target language
    /// Returns a vector of translated strings corresponding to the input.
    /// If translation fails, returns the original strings.
    pub async fn translate_batch(&self, texts: Vec<String>, target_lang: &str) -> Vec<String> {
        if texts.is_empty() {
            return vec![];
        }

        let enabled_str = self.config.get_string("ai_enabled", "false").await;
        let enabled = enabled_str == "true";

        let provider = self.config.get_string("ai_provider", "openai").await;
        let api_key = self.config.get_string("ai_api_key", "").await;
        let base_url = self.config.get_string("ai_base_url", "https://api.openai.com/v1").await;
        let model = self.config.get_string("ai_model", "gpt-4o").await;

        if !enabled {
            return texts;
        }

        if api_key.is_empty() {
            error!("AI Translation skipped: No API Key configured");
            return texts;
        }

        // Prepare prompt
        // We use a simple structured prompt.
        // Input: "Hello", "World"
        // Prompt: Translate the following JSON array of strings to {target_lang}. Return ONLY a JSON array of strings. ["Hello", "World"]
        let prompt = format!(
            "Translate the following array of strings to {}. Return ONLY the raw JSON string array with no markdown formatting. Input: {}",
            target_lang,
            serde_json::to_string(&texts).unwrap_or_default()
        );

        let body = json!({
            "model": model,
            "messages": [
                {
                    "role": "system",
                    "content": "You are a professional translator engine. output strictly JSON array."
                },
                {
                    "role": "user",
                    "content": prompt
                }
            ],
            "temperature": 0.3
        });

        let url = format!("{}/chat/completions", base_url.trim_end_matches('/'));

        info!("Sending translation request to {} ({}) for {} items", provider, model, texts.len());

        let client = Client::build()
            .timeout(std::time::Duration::from_secs(60))
            .finish();

        match client.post(&url)
            .header("Authorization", format!("Bearer {}", api_key))
            .header("Content-Type", "application/json")
            .send_json(&body)
            .await 
        {
            Ok(mut res) => {
                if res.status().is_success() {
                    if let Ok(json_body) = res.json::<Value>().await {
                        // Parse Content
                        if let Some(content) = json_body["choices"][0]["message"]["content"].as_str() {
                            // Try to parse content as JSON array
                            // Clean possible markdown code blocks ```json ... ```
                            let cleaned = content
                                .trim()
                                .trim_start_matches("```json")
                                .trim_start_matches("```")
                                .trim_end_matches("```")
                                .trim();
                                
                            if let Ok(translated) = serde_json::from_str::<Vec<String>>(cleaned) {
                                if translated.len() == texts.len() {
                                    return translated;
                                } else {
                                    error!("AI returned mismatch count: sent {}, got {}", texts.len(), translated.len());
                                }
                            } else {
                                error!("Failed to parse AI response as JSON Array: {}", cleaned);
                            }
                        }
                    }
                } else {
                    error!("AI API Error: {:?}", res.status());
                }
            }
            Err(e) => {
                error!("AI Request Failed: {}", e);
            }
        }

        // Fallback: Return original
        texts
    }
}

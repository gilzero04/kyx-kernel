use std::sync::Arc;
use crate::core::utils::password::{hash_password, verify_password};
use crate::core::AppError;
use crate::modules::system::domain::api_key::{ApiKey, ApiKeyRepository};
use serde_json::Value;
use uuid::Uuid;

pub struct ApiKeyService {
    repo: Arc<dyn ApiKeyRepository>,
}

impl ApiKeyService {
    pub fn new(repo: Arc<dyn ApiKeyRepository>) -> Self {
        Self { repo }
    }

    pub async fn create_key(
        &self,
        tenant_id: &str,
        name: Option<String>,
        key_type: &str,
        allowed_origins: Option<Value>,
    ) -> Result<(ApiKey, String), AppError> {
        let plain_key = format!("sk_live_{}", Uuid::new_v4().to_string().replace("-", ""));
        let key_hash = hash_password(&plain_key)?;
        let prefix = &plain_key[..10]; // sk_live_...

        let key = self.repo.create(tenant_id, &key_hash, prefix, name, key_type, allowed_origins).await
            .map_err(|e| AppError {
                code: 500,
                message: e.to_string(),
            })?;

        Ok((key, plain_key))
    }

    pub async fn verify_key(&self, plain_key: &str, user_agent: Option<&str>, fetch_mode: Option<&str>) -> Result<ApiKey, AppError> {
        if !plain_key.starts_with("sk_live_") {
            return Err(AppError { code: 401, message: "Invalid key format".into() });
        }

        let prefix = &plain_key[..10];
        
        let candidates = self.repo.find_by_prefix(prefix).await
            .map_err(|e| AppError { code: 500, message: e.to_string() })?;

        for key in candidates {
            if verify_password(plain_key, &key.key_hash).unwrap_or(false) {
                // Security Constraints
                if key.key_type == "server" {
                    // Block Browser User-Agents
                    if let Some(ua) = user_agent {
                        let ua_lower = ua.to_lowercase();
                        if ua_lower.contains("mozilla") || fetch_mode.is_some() {
                            return Err(AppError {
                                code: 403,
                                message: "Server API Key cannot be used in a browser".into(),
                            });
                        }
                    }
                }
                return Ok(key);
            }
        }

        Err(AppError { code: 401, message: "Invalid API key".into() })
    }
}

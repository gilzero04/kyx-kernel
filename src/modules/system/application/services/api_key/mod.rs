use crate::core::AppError;
use crate::core::utils::password::{hash_password, verify_password};
use crate::modules::system::domain::api_key::{ApiKey, ApiKeyRepository};
use serde_json::Value;
use std::sync::Arc;
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
        tenant_id: Uuid,
        name: Option<String>,
        key_type: &str,
        allowed_origins: Option<Value>,
    ) -> Result<(ApiKey, String), AppError> {
        // Professional Enterprise Prefixing (Stripe-like)
        // Format: kyx_[pk/sk]_[live/test]_[random_string]
        let prefix_type = match key_type {
            "public" | "client" => "pk",
            "secret" | "server" => "sk",
            _ => "key",
        };

        // Defaulting to 'live' env prefix
        let env_prefix = "live";
        let professional_prefix = format!("kyx_{}_{}_", prefix_type, env_prefix);

        let plain_random = self.generate_secure_random(32);
        let plain_key = format!("{}{}", professional_prefix, plain_random);
        let key_hash = hash_password(&plain_key)?;
        let prefix = &plain_key[..12]; // e.g. kyx_sk_live_

        let key = self
            .repo
            .create(
                tenant_id,
                &key_hash,
                prefix,
                name,
                key_type,
                allowed_origins,
            )
            .await
            .map_err(|e| AppError {
                code: 500,
                message: e.to_string(),
            })?;

        Ok((key, plain_key))
    }

    #[allow(dead_code)]
    pub async fn verify_key(
        &self,
        plain_key: &str,
        user_agent: Option<&str>,
        fetch_mode: Option<&str>,
    ) -> Result<ApiKey, AppError> {
        if !plain_key.starts_with("kyx_") {
            return Err(AppError {
                code: 401,
                message: "Invalid key format".into(),
            });
        }

        let prefix = &plain_key[..12];

        let candidates = self
            .repo
            .find_by_prefix(prefix)
            .await
            .map_err(|e| AppError {
                code: 500,
                message: e.to_string(),
            })?;

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

        Err(AppError {
            code: 401,
            message: "Invalid API key".into(),
        })
    }

    pub async fn list_keys_hierarchical(&self, tenant_id: Uuid) -> Result<Vec<ApiKey>, AppError> {
        self.repo
            .find_hierarchical(tenant_id)
            .await
            .map_err(|e| AppError {
                code: 500,
                message: e.to_string(),
            })
    }

    pub async fn revoke_key(&self, key_id: Uuid, actor_tenant_id: Uuid) -> Result<(), AppError> {
        self.repo
            .delete(key_id, actor_tenant_id)
            .await
            .map_err(|e| AppError {
                code: 400,
                message: e.to_string(),
            })
    }

    fn generate_secure_random(&self, length: usize) -> String {
        use rand::distributions::Alphanumeric;
        use rand::{Rng, thread_rng};

        thread_rng()
            .sample_iter(&Alphanumeric)
            .take(length)
            .map(char::from)
            .collect()
    }
}

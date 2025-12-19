use jsonwebtoken::{encode, decode, Header, Algorithm, Validation, EncodingKey, DecodingKey};
use serde::{Deserialize, Serialize};
use chrono::{Utc, Duration};
use crate::core::AppError;

#[derive(Debug, Serialize, Deserialize, PartialEq)]
#[serde(rename_all = "lowercase")]
pub enum TokenType {
    Access,
    Refresh,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct Claims {
    pub sub: String, // User ID
    pub role: String,
    pub tenant_id: String,
    pub permissions: Vec<String>, // List of permission slugs
    pub exp: usize,
    pub token_type: TokenType,
}

pub struct JwtService {
    encoding_key: EncodingKey,
    decoding_key: DecodingKey,
}

impl JwtService {
    pub fn new(secret: &str) -> Self {
        Self {
            encoding_key: EncodingKey::from_secret(secret.as_bytes()),
            decoding_key: DecodingKey::from_secret(secret.as_bytes()),
        }
    }

    pub fn generate_access_token(&self, user_id: &str, role: &str, tenant_id: &str, permissions: Vec<String>) -> Result<String, AppError> {
        self.generate_token(user_id, role, tenant_id, permissions, TokenType::Access, Duration::minutes(30))
    }

    pub fn generate_refresh_token(&self, user_id: &str, role: &str, tenant_id: &str, permissions: Vec<String>) -> Result<String, AppError> {
        self.generate_token(user_id, role, tenant_id, permissions, TokenType::Refresh, Duration::hours(24))
    }

    pub fn generate_token(
        &self, 
        user_id: &str, 
        role: &str, 
        tenant_id: &str,
        permissions: Vec<String>,
        token_type: TokenType, 
        duration: chrono::Duration
    ) -> Result<String, AppError> {
        let expiration = chrono::Utc::now()
            .checked_add_signed(duration)
            .expect("invalid timestamp")
            .timestamp();

        let claims = Claims {
            sub: user_id.to_owned(),
            role: role.to_owned(),
            tenant_id: tenant_id.to_owned(),
            permissions,
            exp: expiration as usize,
            token_type,
        };

        encode(&Header::default(), &claims, &self.encoding_key)
            .map_err(|e| AppError {
                code: 500,
                message: format!("Token generation failed: {}", e),
            })
    }

    pub fn verify_token(&self, token: &str) -> Result<Claims, AppError> {
        decode::<Claims>(token, &self.decoding_key, &Validation::new(Algorithm::HS256))
            .map(|data| data.claims)
            .map_err(|e| AppError {
                code: 401,
                message: format!("Invalid token: {}", e),
            })
    }
}

use jsonwebtoken::{encode, decode, Header, Algorithm, Validation, EncodingKey, DecodingKey};
use serde::{Deserialize, Serialize};
use chrono::Duration;
use crate::core::AppError;
use uuid::Uuid;
use ntex::web;
use ntex::http::Payload;

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
#[serde(rename_all = "lowercase")]
pub enum TokenType {
    Access,
    Refresh,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Claims {
    pub sub: String, // User ID
    pub role: String,
    pub tenant_id: Uuid,
    pub permissions: Vec<String>, // List of permission slugs
    pub is_system_owner: Option<bool>,
    pub sid: Option<String>, // Session ID
    pub exp: usize,
    pub token_type: TokenType,
}

impl web::FromRequest<web::DefaultError> for Claims {
    type Error = web::Error;

    async fn from_request(req: &web::HttpRequest, _: &mut Payload) -> Result<Self, Self::Error> {
        if let Some(claims) = req.extensions().get::<Claims>() {
            Ok(claims.clone())
        } else {
            // Check if it's a bearer token in header even if middleware didn't run
            let _auth_header = req.headers().get("Authorization");
            // Let middleware handle it or return error
            Err(web::error::ErrorUnauthorized("Unauthorized: Missing or invalid token").into())
        }
    }
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

    pub fn generate_access_token(&self, user_id: &str, role: &str, tenant_id: Uuid, permissions: Vec<String>, is_system_owner: bool, sid: Option<String>) -> Result<String, AppError> {
        self.generate_token(user_id, role, tenant_id, permissions, is_system_owner, TokenType::Access, Duration::minutes(30), sid)
    }

    pub fn generate_refresh_token(&self, user_id: &str, role: &str, tenant_id: Uuid, permissions: Vec<String>, is_system_owner: bool, sid: Option<String>) -> Result<String, AppError> {
        self.generate_token(user_id, role, tenant_id, permissions, is_system_owner, TokenType::Refresh, Duration::hours(24), sid)
    }

    /// Generate access token with custom expiry (in minutes)
    pub fn generate_access_token_dynamic(&self, user_id: &str, role: &str, tenant_id: Uuid, permissions: Vec<String>, is_system_owner: bool, expiry_min: i64, sid: Option<String>) -> Result<String, AppError> {
        self.generate_token(user_id, role, tenant_id, permissions, is_system_owner, TokenType::Access, Duration::minutes(expiry_min), sid)
    }

    /// Generate refresh token with custom expiry (in hours)
    pub fn generate_refresh_token_dynamic(&self, user_id: &str, role: &str, tenant_id: Uuid, permissions: Vec<String>, is_system_owner: bool, expiry_hours: i64, sid: Option<String>) -> Result<String, AppError> {
        self.generate_token(user_id, role, tenant_id, permissions, is_system_owner, TokenType::Refresh, Duration::hours(expiry_hours), sid)
    }

    pub fn generate_token(
        &self, 
        user_id: &str, 
        role: &str, 
        tenant_id: Uuid,
        permissions: Vec<String>,
        is_system_owner: bool,
        token_type: TokenType, 
        duration: chrono::Duration,
        sid: Option<String>,
    ) -> Result<String, AppError> {
        let expiration = chrono::Utc::now()
            .checked_add_signed(duration)
            .expect("invalid timestamp")
            .timestamp();

        let claims = Claims {
            sub: user_id.to_owned(),
            role: role.to_owned(),
            tenant_id,
            permissions,
            is_system_owner: Some(is_system_owner),
            sid,
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

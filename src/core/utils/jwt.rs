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
    
    // Phase 2: Plan-specific claims
    /// User's plan type (free, pro, enterprise)
    #[serde(skip_serializing_if = "Option::is_none")]
    pub plan_type: Option<String>,
    
    /// Feature flags enabled for this user/tenant
    #[serde(skip_serializing_if = "Option::is_none")]
    pub features: Option<PlanFeatures>,
    
    // Phase 2: Signal-specific claims (for kyx-signal integration)
    /// Mapped signal permissions (subscribe, publish, join, etc.)
    #[serde(skip_serializing_if = "Option::is_none")]
    pub signal_permissions: Option<Vec<String>>,
}

/// Plan-based feature flags and rate limits
#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct PlanFeatures {
    /// Maximum WebSocket connections
    #[serde(default)]
    pub max_connections: u32,
    
    /// Messages per minute rate limit
    #[serde(default)]
    pub messages_per_minute: u32,
    
    /// Maximum rooms user can join
    #[serde(default)]
    pub max_rooms: u32,
    
    /// AI features enabled (face search, etc.)
    #[serde(default)]
    pub ai_enabled: bool,
    
    /// Custom domain enabled
    #[serde(default)]
    pub custom_domain: bool,
    
    /// White-label branding
    #[serde(default)]
    pub white_label: bool,
}

impl PlanFeatures {
    /// Create features for a specific plan type
    pub fn for_plan(plan_type: &str) -> Self {
        match plan_type {
            "free" => Self {
                max_connections: 2,
                messages_per_minute: 30,
                max_rooms: 5,
                ai_enabled: false,
                custom_domain: false,
                white_label: false,
            },
            "pro" => Self {
                max_connections: 10,
                messages_per_minute: 200,
                max_rooms: 50,
                ai_enabled: true,
                custom_domain: true,
                white_label: false,
            },
            "enterprise" => Self {
                max_connections: 100,
                messages_per_minute: 1000,
                max_rooms: 500,
                ai_enabled: true,
                custom_domain: true,
                white_label: true,
            },
            _ => Self::default(),
        }
    }
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

    /// Get encoding key for generating tokens (e.g., signal tickets)
    pub fn get_encoding_key(&self) -> &EncodingKey {
        &self.encoding_key
    }

    pub fn generate_access_token(&self, user_id: &str, role: &str, tenant_id: Uuid, permissions: Vec<String>, is_system_owner: bool, sid: Option<String>) -> Result<String, AppError> {
        self.generate_token(user_id, role, tenant_id, permissions, is_system_owner, TokenType::Access, Duration::minutes(30), sid)
    }

    pub fn generate_refresh_token(&self, user_id: &str, role: &str, tenant_id: Uuid, permissions: Vec<String>, is_system_owner: bool, sid: Option<String>) -> Result<String, AppError> {
        self.generate_token(user_id, role, tenant_id, permissions, is_system_owner, TokenType::Refresh, Duration::hours(24), sid)
    }

    /// Generate access token with plan information (for kyx-signal integration)
    pub fn generate_access_token_with_plan(
        &self,
        user_id: &str,
        role: &str,
        tenant_id: Uuid,
        permissions: Vec<String>,
        is_system_owner: bool,
        sid: Option<String>,
        plan_type: &str,
    ) -> Result<String, AppError> {
        let expiration = chrono::Utc::now()
            .checked_add_signed(Duration::minutes(30))
            .expect("invalid timestamp")
            .timestamp();

        // Map kernel permissions to signal permissions
        let signal_perms = permissions.iter()
            .filter_map(|p| match p.as_str() {
                "chat:read" => Some("subscribe".to_string()),
                "chat:send" => Some("publish".to_string()),
                "room:join" => Some("join".to_string()),
                "room:admin" => Some("moderate".to_string()),
                _ => None,
            })
            .collect::<Vec<_>>();

        let claims = Claims {
            sub: user_id.to_owned(),
            role: role.to_owned(),
            tenant_id,
            permissions,
            is_system_owner: Some(is_system_owner),
            sid,
            exp: expiration as usize,
            token_type: TokenType::Access,
            plan_type: Some(plan_type.to_string()),
            features: Some(PlanFeatures::for_plan(plan_type)),
            signal_permissions: if signal_perms.is_empty() { None } else { Some(signal_perms) },
        };

        encode(&Header::default(), &claims, &self.encoding_key)
            .map_err(|e| AppError {
                code: 500,
                message: format!("Token generation failed: {}", e),
            })
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
            // Phase 2: Optional fields (None for backward compatibility)
            plan_type: None,
            features: None,
            signal_permissions: None,
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

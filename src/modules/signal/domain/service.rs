use std::sync::Arc;
use anyhow::Result;
use uuid::Uuid;
use jsonwebtoken::{encode, Header};

use crate::core::utils::jwt::JwtService;
use super::super::domain::entity::{SignalTicketClaims, PermissionMapper};

/// Service for generating signal tickets
pub struct SignalService {
    jwt: Arc<JwtService>,
    signal_url: String,
    default_ttl: i64,
}

/// Response from signal ticket generation
#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
pub struct SignalTicketResponse {
    pub ticket: String,
    pub expires_at: String,
    pub signal_url: String,
}

impl SignalService {
    pub fn new(jwt: Arc<JwtService>) -> Self {
        let signal_url = std::env::var("SIGNAL_WS_URL")
            .unwrap_or_else(|_| "wss://localhost:8081/ws".to_string());
        
        Self {
            jwt,
            signal_url,
            default_ttl: 60, // 60 seconds default
        }
    }

    /// Generate a signal ticket for the given user
    pub fn generate_ticket(
        &self,
        user_id: Uuid,
        tenant_id: Uuid,
        kernel_permissions: &[String],
        plan: &str,
        ttl: Option<i64>,
    ) -> Result<SignalTicketResponse> {
        let signal_permissions = PermissionMapper::map(kernel_permissions);
        
        // Check if user has any signal permissions
        if signal_permissions.is_empty() {
            return Err(anyhow::anyhow!("User has no signal permissions"));
        }

        let ttl_seconds = ttl.unwrap_or(self.default_ttl);
        let claims = SignalTicketClaims::new(
            user_id,
            tenant_id,
            signal_permissions,
            plan.to_string(),
            ttl_seconds,
        );

        // Use JwtService's encoding key
        let token = encode(
            &Header::default(),
            &claims,
            self.jwt.get_encoding_key(),
        )?;

        let expires_at = chrono::Utc::now() + chrono::Duration::seconds(ttl_seconds);

        Ok(SignalTicketResponse {
            ticket: token,
            expires_at: expires_at.to_rfc3339(),
            signal_url: self.signal_url.clone(),
        })
    }
}

use ntex::web;
use serde::{Deserialize, Serialize};
use std::sync::Arc;
use utoipa::ToSchema;

use crate::core::utils::jwt::{Claims, JwtService};
use crate::modules::signal::domain::service::SignalService;

/// Request body for signal token generation
#[derive(Debug, Deserialize, ToSchema)]
pub struct SignalTokenRequest {
    /// Optional: Restrict to specific rooms
    #[serde(default)]
    pub rooms: Vec<String>,
    /// Optional: Custom TTL in seconds (max 300, default 60)
    pub ttl: Option<i64>,
}

/// Response for signal token generation
#[derive(Debug, Serialize, ToSchema)]
pub struct SignalTokenResponse {
    /// The signal ticket JWT
    pub ticket: String,
    /// ISO 8601 expiration timestamp
    pub expires_at: String,
    /// WebSocket URL to connect to
    pub signal_url: String,
}

/// Error response
#[derive(Debug, Serialize, ToSchema)]
pub struct SignalErrorResponse {
    pub error: String,
    pub message: String,
}

/// Generate a signal ticket for WebSocket connection
///
/// This endpoint issues a short-lived JWT token for connecting to kyx-signal.
/// The ticket is valid for 30-60 seconds and contains mapped permissions.
#[utoipa::path(
    post,
    path = "/api/v1/signal/token",
    request_body = SignalTokenRequest,
    responses(
        (status = 200, description = "Signal ticket generated", body = SignalTokenResponse),
        (status = 401, description = "Unauthorized", body = SignalErrorResponse),
        (status = 403, description = "No signal permissions", body = SignalErrorResponse),
    ),
    tag = "signal",
    security(("bearer_auth" = []))
)]
pub async fn generate_signal_token(
    jwt: web::types::State<Arc<JwtService>>,
    claims: Claims,
    body: web::types::Json<SignalTokenRequest>,
) -> web::HttpResponse {
    // Parse user_id from String to Uuid
    let user_id = match uuid::Uuid::parse_str(&claims.sub) {
        Ok(id) => id,
        Err(_) => {
            return web::HttpResponse::BadRequest().json(&SignalErrorResponse {
                error: "invalid_user_id".to_string(),
                message: "Invalid user ID format".to_string(),
            });
        }
    };

    // Validate TTL (max 300 seconds = 5 minutes)
    let ttl = body.ttl.map(|t| t.clamp(10, 300));

    // Get plan and features from claims (populated by kyx-plan plugin if installed)
    let plan = claims.plan_type.clone();
    let features = claims.features.as_ref();

    // Create signal service
    let signal_service = SignalService::new(jwt.get_ref().clone());

    // Generate ticket
    match signal_service.generate_ticket(
        user_id,
        claims.tenant_id,
        &claims.permissions,
        plan,
        features,
        ttl,
    ) {
        Ok(response) => {
            log::info!(
                "Signal ticket generated for user {} tenant {}",
                user_id,
                claims.tenant_id
            );
            web::HttpResponse::Ok().json(&SignalTokenResponse {
                ticket: response.ticket,
                expires_at: response.expires_at,
                signal_url: response.signal_url,
            })
        }
        Err(e) => {
            log::warn!("Failed to generate signal ticket: {}", e);
            web::HttpResponse::Forbidden().json(&SignalErrorResponse {
                error: "forbidden".to_string(),
                message: e.to_string(),
            })
        }
    }
}

/// Health check for signal integration
#[utoipa::path(
    get,
    path = "/api/v1/signal/health",
    responses(
        (status = 200, description = "Signal integration healthy"),
    ),
    tag = "signal"
)]
pub async fn signal_health() -> web::HttpResponse {
    web::HttpResponse::Ok().json(&serde_json::json!({
        "status": "ok",
        "service": "kyx-signal-integration"
    }))
}

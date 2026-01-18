use ntex::service::{Middleware, Service, ServiceCtx};
use ntex::web;
use std::collections::HashMap;
use std::sync::Arc;
use std::time::{Duration, Instant};
use tokio::sync::RwLock;

/// Rate limit configuration
#[derive(Clone)]
pub struct RateLimitConfig {
    /// Maximum requests allowed in the window
    pub max_requests: u32,
    /// Time window in seconds
    pub window_secs: u64,
}

impl Default for RateLimitConfig {
    fn default() -> Self {
        Self {
            max_requests: 100, // 100 requests
            window_secs: 60,   // per minute
        }
    }
}

/// In-memory rate limit store
struct RateLimitEntry {
    count: u32,
    window_start: Instant,
}

/// Rate Limiting Middleware
pub struct RateLimit {
    config: RateLimitConfig,
    store: Arc<RwLock<HashMap<String, RateLimitEntry>>>,
}

impl RateLimit {
    #[allow(dead_code)]
    pub fn new(config: RateLimitConfig) -> Self {
        Self {
            config,
            store: Arc::new(RwLock::new(HashMap::new())),
        }
    }

    /// Strict rate limit for sensitive endpoints (login, register)
    #[allow(dead_code)]
    pub fn strict() -> Self {
        Self::new(RateLimitConfig {
            max_requests: 10, // 10 requests
            window_secs: 60,  // per minute
        })
    }

    /// Default rate limit for general API
    #[allow(dead_code)]
    pub fn default_limit() -> Self {
        Self::new(RateLimitConfig::default())
    }
}

impl<S> Middleware<S> for RateLimit {
    type Service = RateLimitMiddleware<S>;

    fn create(&self, service: S) -> Self::Service {
        RateLimitMiddleware {
            service,
            config: self.config.clone(),
            store: self.store.clone(),
        }
    }
}

pub struct RateLimitMiddleware<S> {
    service: S,
    config: RateLimitConfig,
    store: Arc<RwLock<HashMap<String, RateLimitEntry>>>,
}

impl<S, Err> Service<web::WebRequest<Err>> for RateLimitMiddleware<S>
where
    S: Service<web::WebRequest<Err>, Response = web::WebResponse, Error = web::Error>,
{
    type Response = web::WebResponse;
    type Error = web::Error;

    ntex::forward_ready!(service);

    async fn call(
        &self,
        req: web::WebRequest<Err>,
        ctx: ServiceCtx<'_, Self>,
    ) -> Result<Self::Response, Self::Error> {
        // Get client identifier (IP from connection info or peer address)
        let client_id = req
            .peer_addr()
            .map(|addr| addr.to_string())
            .unwrap_or_else(|| "unknown".to_string());

        let now = Instant::now();
        let window_duration = Duration::from_secs(self.config.window_secs);

        // Check rate limit
        let should_block = {
            let mut store = self.store.write().await;

            if let Some(entry) = store.get_mut(&client_id) {
                if now.duration_since(entry.window_start) > window_duration {
                    // Reset window
                    entry.count = 1;
                    entry.window_start = now;
                    false
                } else {
                    entry.count += 1;
                    entry.count > self.config.max_requests
                }
            } else {
                // New client
                store.insert(
                    client_id.clone(),
                    RateLimitEntry {
                        count: 1,
                        window_start: now,
                    },
                );
                false
            }
        };

        if should_block {
            return Ok(req.into_response(
                web::HttpResponse::TooManyRequests()
                    .set_header("Retry-After", self.config.window_secs.to_string())
                    .json(&serde_json::json!({
                        "status": "error",
                        "message": "Too many requests. Please try again later.",
                        "retry_after": self.config.window_secs
                    })),
            ));
        }

        // Add rate limit headers to response
        let mut response = ctx.call(&self.service, req).await?;
        
        // Add rate limit headers
        response.headers_mut().insert(
            ntex::http::header::HeaderName::from_static("x-ratelimit-limit"),
            ntex::http::header::HeaderValue::from_str(&self.config.max_requests.to_string())
                .unwrap_or_else(|_| ntex::http::header::HeaderValue::from_static("0")),
        );
        
        Ok(response)
    }
}

// ==========================================
// Dynamic Rate Limit (reads from ConfigService)
// ==========================================

use crate::core::infrastructure::config_service::ConfigService;

/// Dynamic Rate Limiting that reads from ConfigService
pub struct DynamicRateLimit {
    config_service: Arc<ConfigService>,
    store: Arc<RwLock<HashMap<String, RateLimitEntry>>>,
}

impl DynamicRateLimit {
    pub fn new(config_service: Arc<ConfigService>) -> Self {
        Self {
            config_service,
            store: Arc::new(RwLock::new(HashMap::new())),
        }
    }
}

impl<S> Middleware<S> for DynamicRateLimit {
    type Service = DynamicRateLimitMiddleware<S>;

    fn create(&self, service: S) -> Self::Service {
        DynamicRateLimitMiddleware {
            service,
            config_service: self.config_service.clone(),
            store: self.store.clone(),
        }
    }
}

pub struct DynamicRateLimitMiddleware<S> {
    service: S,
    config_service: Arc<ConfigService>,
    store: Arc<RwLock<HashMap<String, RateLimitEntry>>>,
}

impl<S, Err> Service<web::WebRequest<Err>> for DynamicRateLimitMiddleware<S>
where
    S: Service<web::WebRequest<Err>, Response = web::WebResponse, Error = web::Error>,
{
    type Response = web::WebResponse;
    type Error = web::Error;

    ntex::forward_ready!(service);

    async fn call(
        &self,
        req: web::WebRequest<Err>,
        ctx: ServiceCtx<'_, Self>,
    ) -> Result<Self::Response, Self::Error> {
        // Read config dynamically from ConfigService
        let max_requests = self
            .config_service
            .get_int("rate_limit_max_requests", 100)
            .await as u32;
        let window_secs = self
            .config_service
            .get_int("rate_limit_window_secs", 60)
            .await as u64;

        // Get client identifier
        let client_id = req
            .peer_addr()
            .map(|addr| addr.to_string())
            .unwrap_or_else(|| "unknown".to_string());

        let now = Instant::now();
        let window_duration = Duration::from_secs(window_secs);

        // Check rate limit
        let should_block = {
            let mut store = self.store.write().await;

            if let Some(entry) = store.get_mut(&client_id) {
                if now.duration_since(entry.window_start) > window_duration {
                    entry.count = 1;
                    entry.window_start = now;
                    false
                } else {
                    entry.count += 1;
                    entry.count > max_requests
                }
            } else {
                store.insert(
                    client_id.clone(),
                    RateLimitEntry {
                        count: 1,
                        window_start: now,
                    },
                );
                false
            }
        };

        if should_block {
            return Ok(req.into_response(
                web::HttpResponse::TooManyRequests()
                    .set_header("Retry-After", window_secs.to_string())
                    .json(&serde_json::json!({
                        "status": "error",
                        "message": "Too many requests. Please try again later.",
                        "retry_after": window_secs
                    })),
            ));
        }

        ctx.call(&self.service, req).await
    }
}

// ==========================================
// Plan-Aware Rate Limit (reads from JWT claims)
// Phase 3: Multi-layer rate limiting
// Features should come from kyx-plan plugin via Claims
// ==========================================

use crate::core::utils::jwt::Claims;

// Default rate limits when no plan plugin is installed
const DEFAULT_MAX_CONNECTIONS: u32 = 100;
const DEFAULT_MESSAGES_PER_MINUTE: u32 = 100;

/// Helper to extract u32 from serde_json::Value
fn get_u32_from_json(value: &serde_json::Value, key: &str, default: u32) -> u32 {
    value
        .get(key)
        .and_then(|v| v.as_u64())
        .map(|v| v as u32)
        .unwrap_or(default)
}

/// Helper to extract bool from serde_json::Value
fn get_bool_from_json(value: &serde_json::Value, key: &str, default: bool) -> bool {
    value.get(key).and_then(|v| v.as_bool()).unwrap_or(default)
}

/// Rate limit key combining user, tenant, and endpoint
#[derive(Hash, Eq, PartialEq, Clone)]
struct PlanRateLimitKey {
    tenant_id: String,
    user_id: String,
    endpoint_category: String,
}

/// Plan-aware rate limiting that reads limits from JWT claims
/// Supports multi-layer limiting: per-plan, per-tenant, per-endpoint
pub struct PlanAwareRateLimit {
    store: Arc<RwLock<HashMap<String, RateLimitEntry>>>,
    endpoint_category: String,
}

impl PlanAwareRateLimit {
    pub fn new(endpoint_category: impl Into<String>) -> Self {
        Self {
            store: Arc::new(RwLock::new(HashMap::new())),
            endpoint_category: endpoint_category.into(),
        }
    }

    /// For signal endpoints (WebSocket connections, messaging)
    #[allow(dead_code)]
    pub fn signal() -> Self {
        Self::new("signal")
    }

    /// For AI endpoints (face search, etc.)
    #[allow(dead_code)]
    pub fn ai() -> Self {
        Self::new("ai")
    }

    /// For general API endpoints
    #[allow(dead_code)]
    pub fn api() -> Self {
        Self::new("api")
    }
}

impl<S> Middleware<S> for PlanAwareRateLimit {
    type Service = PlanAwareRateLimitMiddleware<S>;

    fn create(&self, service: S) -> Self::Service {
        PlanAwareRateLimitMiddleware {
            service,
            store: self.store.clone(),
            endpoint_category: self.endpoint_category.clone(),
        }
    }
}

pub struct PlanAwareRateLimitMiddleware<S> {
    service: S,
    store: Arc<RwLock<HashMap<String, RateLimitEntry>>>,
    endpoint_category: String,
}

impl<S, Err> Service<web::WebRequest<Err>> for PlanAwareRateLimitMiddleware<S>
where
    S: Service<web::WebRequest<Err>, Response = web::WebResponse, Error = web::Error>,
{
    type Response = web::WebResponse;
    type Error = web::Error;

    ntex::forward_ready!(service);

    async fn call(
        &self,
        req: web::WebRequest<Err>,
        ctx: ServiceCtx<'_, Self>,
    ) -> Result<Self::Response, Self::Error> {
        // Try to get claims from request extensions
        let (max_requests, window_secs, rate_limit_key) = {
            let extensions = req.extensions();

            if let Some(claims) = extensions.get::<Claims>() {
                // Parse features from JSON Value (populated by kyx-plan plugin)
                // If no features, use defaults (no rate limiting by plan)
                let (messages_per_minute, ai_enabled) = if let Some(ref features) = claims.features
                {
                    (
                        get_u32_from_json(
                            features,
                            "messages_per_minute",
                            DEFAULT_MESSAGES_PER_MINUTE,
                        ),
                        get_bool_from_json(features, "ai_enabled", true),
                    )
                } else {
                    // No plan plugin installed - use generous defaults
                    (DEFAULT_MESSAGES_PER_MINUTE, true)
                };

                // Determine max requests based on endpoint category
                let max_req = match self.endpoint_category.as_str() {
                    "signal" => messages_per_minute,
                    "ai" => {
                        if ai_enabled {
                            20
                        } else {
                            0
                        }
                    }
                    _ => messages_per_minute.max(100),
                };

                // Create composite key: tenant + user + endpoint
                let key = format!(
                    "{}:{}:{}",
                    claims.tenant_id, claims.sub, self.endpoint_category
                );

                (max_req, 60u64, key)
            } else {
                // No claims - use IP-based limiting with default limits
                let ip = req
                    .peer_addr()
                    .map(|addr| addr.to_string())
                    .unwrap_or_else(|| "unknown".to_string());

                let key = format!("anon:{}:{}", ip, self.endpoint_category);
                (DEFAULT_MESSAGES_PER_MINUTE, 60u64, key)
            }
        };

        // Check if this endpoint category is disabled for the plan
        if max_requests == 0 {
            return Ok(req.into_response(web::HttpResponse::Forbidden().json(
                &serde_json::json!({
                    "status": "error",
                    "message": "This feature is not available on your plan.",
                    "upgrade_required": true
                }),
            )));
        }

        let now = Instant::now();
        let window_duration = Duration::from_secs(window_secs);

        // Check rate limit
        let (should_block, remaining, reset_after) = {
            let mut store = self.store.write().await;

            if let Some(entry) = store.get_mut(&rate_limit_key) {
                let elapsed = now.duration_since(entry.window_start);
                if elapsed > window_duration {
                    // Reset window
                    entry.count = 1;
                    entry.window_start = now;
                    (false, max_requests - 1, window_secs)
                } else {
                    entry.count += 1;
                    let blocked = entry.count > max_requests;
                    let remaining = if blocked {
                        0
                    } else {
                        max_requests - entry.count
                    };
                    let reset = window_secs - elapsed.as_secs();
                    (blocked, remaining, reset)
                }
            } else {
                // New key
                store.insert(
                    rate_limit_key.clone(),
                    RateLimitEntry {
                        count: 1,
                        window_start: now,
                    },
                );
                (false, max_requests - 1, window_secs)
            }
        };

        if should_block {
            return Ok(req.into_response(
                web::HttpResponse::TooManyRequests()
                    .set_header("X-RateLimit-Limit", max_requests.to_string())
                    .set_header("X-RateLimit-Remaining", "0")
                    .set_header("X-RateLimit-Reset", reset_after.to_string())
                    .set_header("Retry-After", reset_after.to_string())
                    .json(&serde_json::json!({
                        "status": "error",
                        "message": "Rate limit exceeded. Please try again later.",
                        "retry_after": reset_after,
                        "limit": max_requests,
                        "category": self.endpoint_category
                    })),
            ));
        }

        // Call the underlying service
        let mut response = ctx.call(&self.service, req).await?;

        // Add rate limit headers to response
        response.headers_mut().insert(
            ntex::http::header::HeaderName::from_static("x-ratelimit-limit"),
            ntex::http::header::HeaderValue::from_str(&max_requests.to_string())
                .unwrap_or_else(|_| ntex::http::header::HeaderValue::from_static("0")),
        );
        response.headers_mut().insert(
            ntex::http::header::HeaderName::from_static("x-ratelimit-remaining"),
            ntex::http::header::HeaderValue::from_str(&remaining.to_string())
                .unwrap_or_else(|_| ntex::http::header::HeaderValue::from_static("0")),
        );
        response.headers_mut().insert(
            ntex::http::header::HeaderName::from_static("x-ratelimit-reset"),
            ntex::http::header::HeaderValue::from_str(&reset_after.to_string())
                .unwrap_or_else(|_| ntex::http::header::HeaderValue::from_static("0")),
        );

        Ok(response)
    }
}

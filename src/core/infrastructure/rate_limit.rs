use ntex::service::{Middleware, Service, ServiceCtx};
use ntex::web;
use std::sync::Arc;
use std::collections::HashMap;
use tokio::sync::RwLock;
use std::time::{Duration, Instant};

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
            max_requests: 100,  // 100 requests
            window_secs: 60,    // per minute
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
    pub fn new(config: RateLimitConfig) -> Self {
        Self {
            config,
            store: Arc::new(RwLock::new(HashMap::new())),
        }
    }

    /// Strict rate limit for sensitive endpoints (login, register)
    pub fn strict() -> Self {
        Self::new(RateLimitConfig {
            max_requests: 10,   // 10 requests
            window_secs: 60,    // per minute
        })
    }

    /// Default rate limit for general API
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

    async fn call(&self, req: web::WebRequest<Err>, ctx: ServiceCtx<'_, Self>) -> Result<Self::Response, Self::Error> {
        // Get client identifier (IP from connection info or peer address)
        let client_id = req.peer_addr()
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
                store.insert(client_id.clone(), RateLimitEntry {
                    count: 1,
                    window_start: now,
                });
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
                    }))
            ));
        }

        // Add rate limit headers to response
        let response = ctx.call(&self.service, req).await?;
        
        // TODO: Add X-RateLimit-Remaining header
        Ok(response)
    }
}

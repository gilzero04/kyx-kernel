use ntex::web;
use std::sync::Arc;

use super::handlers;
use crate::core::infrastructure::audit::AuditService;
use crate::core::infrastructure::permission_middleware::RequirePermission;
use crate::core::infrastructure::redis::Redis;
use crate::core::utils::jwt::JwtService;

pub fn signal_routes(
    config: &mut web::ServiceConfig,
    jwt: Arc<JwtService>,
    audit: Arc<AuditService>,
    redis: Option<Arc<Redis>>,
) {
    config.service(
        web::scope("/signal")
            // Public health check
            .route("/health", web::get().to(handlers::signal_health))
            // Protected token generation
            .service(
                web::resource("/token")
                    .wrap(RequirePermission::new("chat:read", jwt, audit).with_redis_opt(redis))
                    .route(web::post().to(handlers::generate_signal_token)),
            ),
    );
}

use crate::modules::system::interface::http::handlers::share;
use ntex::web;

pub fn share_routes(
    config: &mut web::ServiceConfig,
    jwt: std::sync::Arc<crate::core::utils::jwt::JwtService>,
    audit: std::sync::Arc<crate::core::infrastructure::audit::AuditService>,
    redis: Option<std::sync::Arc<crate::core::infrastructure::redis::Redis>>,
) {
    use crate::core::infrastructure::permission_middleware::RequirePermission;

    // Resource Sharing endpoints - require system:read permission
    config.service(
        web::scope("/shares")
            .wrap(
                RequirePermission::new("system:read", jwt.clone(), audit.clone())
                    .with_redis_opt(redis.clone()),
            )
            .route("", web::get().to(share::list_shares))
            .route("/received", web::get().to(share::list_received_shares))
            .service(
                web::resource("")
                    .wrap(
                        RequirePermission::new("system:manage", jwt.clone(), audit.clone())
                            .with_redis_opt(redis.clone()),
                    )
                    .route(web::post().to(share::create_share)),
            )
            .service(
                web::scope("/{id}")
                    .route("/usage", web::get().to(share::get_share_usage))
                    .service(
                        web::resource("")
                            .guard(web::guard::Delete())
                            .wrap(
                                RequirePermission::new("system:manage", jwt.clone(), audit.clone())
                                    .with_redis_opt(redis),
                            )
                            .route(web::delete().to(share::revoke_share)),
                    ),
            ),
    );
}

// Keep the old configure function for backward compatibility
pub fn configure(cfg: &mut web::ServiceConfig) {
    // Resource Sharing endpoints (requires auth from parent scope)
    cfg.service(
        web::scope("/shares")
            .route("", web::get().to(share::list_shares))
            .route("", web::post().to(share::create_share))
            .route("/received", web::get().to(share::list_received_shares))
            .route("/{id}", web::delete().to(share::revoke_share))
            .route("/{id}/usage", web::get().to(share::get_share_usage)),
    );
}

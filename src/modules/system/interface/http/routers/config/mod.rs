use ntex::web;
use crate::modules::system::interface::http::handlers::config;

pub fn config_routes(
    cfg: &mut web::ServiceConfig, 
    jwt: std::sync::Arc<crate::core::utils::jwt::JwtService>, 
    audit: std::sync::Arc<crate::core::infrastructure::audit::AuditService>,
    redis: Option<std::sync::Arc<crate::core::infrastructure::redis::Redis>>,
) {
    use crate::core::infrastructure::permission_middleware::RequirePermission;

    // Register at /config
    cfg.service(
        web::scope("/config")
            .service(
                web::resource("")
                    .guard(web::guard::Get())
                    .route(web::get().to(config::get_config))
                    .wrap(RequirePermission::new("system:config:read", jwt.clone(), audit.clone()).with_redis_opt(redis.clone()))
            )
            .service(
                web::resource("")
                    .guard(web::guard::Patch())
                    .route(web::patch().to(config::update_config))
                    .wrap(RequirePermission::new("system:config:update", jwt.clone(), audit.clone()).with_redis_opt(redis.clone()))
            )
    );

    // Also register at /settings for frontend compatibility
    cfg.service(
        web::scope("/settings")
            .service(
                web::resource("")
                    .guard(web::guard::Get())
                    .route(web::get().to(config::get_config))
                    .wrap(RequirePermission::new("system:config:read", jwt.clone(), audit.clone()).with_redis_opt(redis.clone()))
            )
            .service(
                web::resource("")
                    .guard(web::guard::Patch())
                    .route(web::patch().to(config::update_config))
                    .wrap(RequirePermission::new("system:config:update", jwt.clone(), audit.clone()).with_redis_opt(redis))
            )
    );
}

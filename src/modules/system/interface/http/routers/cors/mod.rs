use ntex::web;
use crate::modules::system::interface::http::handlers::cors;

pub fn cors_routes(
    config: &mut web::ServiceConfig, 
    jwt: std::sync::Arc<crate::core::utils::jwt::JwtService>, 
    audit: std::sync::Arc<crate::core::infrastructure::audit::AuditService>,
    redis: Option<std::sync::Arc<crate::core::infrastructure::redis::Redis>>,
) {
    use crate::core::infrastructure::permission_middleware::RequirePermission;

    config.service(
        web::scope("/cors")
            .wrap(RequirePermission::new("system:cors:manage", jwt.clone(), audit.clone()).with_redis_opt(redis))
            .route("", web::get().to(cors::list_cors_origins))
            .route("", web::post().to(cors::add_cors_origin))
            .route("/{id}", web::patch().to(cors::update_cors_origin))
            .route("/{id}", web::delete().to(cors::delete_cors_origin))
    );
}

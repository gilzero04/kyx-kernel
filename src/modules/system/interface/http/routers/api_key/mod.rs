use ntex::web;
use crate::modules::system::interface::http::handlers::api_key;

pub fn api_key_routes(
    config: &mut web::ServiceConfig, 
    jwt: std::sync::Arc<crate::core::utils::jwt::JwtService>, 
    audit: std::sync::Arc<crate::core::infrastructure::audit::AuditService>,
    redis: Option<std::sync::Arc<crate::core::infrastructure::redis::Redis>>,
) {
    use crate::core::infrastructure::permission_middleware::RequirePermission;

    config.service(
        web::scope("/api-keys")
            .wrap(RequirePermission::new("system:api_key:manage", jwt.clone(), audit.clone()).with_redis_opt(redis))
            .route("", web::post().to(api_key::create_api_key))
            .route("", web::get().to(api_key::list_api_keys))
            .route("/{id}", web::delete().to(api_key::revoke_api_key))
    );
}

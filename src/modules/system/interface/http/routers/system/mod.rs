use ntex::web;
use crate::modules::system::interface::http::handlers::system;
use ntex::web::DefaultError;

pub fn public_routes() -> web::Scope<DefaultError> {
    web::scope("/status")
        .route("", web::get().to(system::get_system_status))
}

#[allow(dead_code)]
pub fn tenant_routes(_config: &mut web::ServiceConfig, _jwt: std::sync::Arc<crate::core::utils::jwt::JwtService>, _audit: std::sync::Arc<crate::core::infrastructure::audit::AuditService>) {

    // The following lines are commented out because the parameters _config, _jwt, _audit
    // are intentionally unused in the function signature to suppress warnings,
    // and using them here would require removing the '_' prefix, which might
    // contradict the intent of the `#[allow(dead_code)]` attribute on the function itself.
    // If these routes are intended to be active, the '_' prefixes should be removed
    // from the function parameters and the following lines uncommented.

    /*
    _config.service(
        web::resource("/settings")
            .wrap(RequirePermission::new("system:read", _jwt.clone(), _audit.clone()))
            .route(web::get().to(system::get_system_settings))
    );

    _config.service(
        web::resource("/context")
            .wrap(RequirePermission::new("system:read", _jwt.clone(), _audit.clone()))
            .route(web::get().to(system::get_system_context))
    );

    _config.service(
        web::resource("/test")
            .wrap(RequirePermission::new("system:read", _jwt, _audit))
            .route(web::get().to(system::admin_test))
    );
    */
}

#[allow(dead_code)]
pub fn admin_routes(
    config: &mut web::ServiceConfig, 
    jwt: std::sync::Arc<crate::core::utils::jwt::JwtService>, 
    audit: std::sync::Arc<crate::core::infrastructure::audit::AuditService>,
    redis: Option<std::sync::Arc<crate::core::infrastructure::redis::Redis>>,
) {
    use crate::core::infrastructure::permission_middleware::RequirePermission;

    config.service(
        web::resource("/settings")
            .wrap(RequirePermission::new("system:read", jwt.clone(), audit.clone()).with_redis_opt(redis.clone()))
            .route(web::get().to(system::get_system_settings))
    );

    config.service(
        web::resource("/context")
            .wrap(RequirePermission::new("system:read", jwt.clone(), audit.clone()).with_redis_opt(redis.clone()))
            .route(web::get().to(system::get_system_context))
    );

    config.service(
        web::resource("/test")
            .wrap(RequirePermission::new("system:read", jwt, audit).with_redis_opt(redis))
            .route(web::get().to(system::admin_test))
    );
}

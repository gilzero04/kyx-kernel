use ntex::web;
use crate::modules::system::interface::http::handlers::tenant;

pub fn tenant_routes_system(
    cfg: &mut web::ServiceConfig, 
    jwt: std::sync::Arc<crate::core::utils::jwt::JwtService>, 
    audit: std::sync::Arc<crate::core::infrastructure::audit::AuditService>,
    redis: Option<std::sync::Arc<crate::core::infrastructure::redis::Redis>>,
) {
    use crate::core::infrastructure::permission_middleware::RequirePermission;

    cfg.service(
        web::scope("/tenants")
            .wrap(RequirePermission::new("", jwt.clone(), audit.clone()).with_redis_opt(redis.clone()))
            // Self-service route (Authenticated)
            .route("/me", web::get().to(tenant::get_current_tenant))
            
            // Admin operations (Protected)
            // List Tenants
            .service(
                web::resource("")
                    .wrap(RequirePermission::system("tenant:read", jwt.clone(), audit.clone()).with_redis_opt(redis.clone()))
                    .route(web::get().to(tenant::list_tenants))
                    .wrap(RequirePermission::system("tenant:create", jwt.clone(), audit.clone()).with_redis_opt(redis.clone()))
                    .route(web::post().to(tenant::create_tenant))
            )
            // Owner Branding
            .service(
                web::resource("/owner")
                    .wrap(RequirePermission::system("tenant:update", jwt.clone(), audit.clone()).with_redis_opt(redis.clone()))
                    .route(web::patch().to(tenant::update_owner))
            )
            // Individual Tenant Operations
            .service(
                web::resource("/{id}")
                    .wrap(RequirePermission::system("tenant:read", jwt.clone(), audit.clone()).with_redis_opt(redis.clone()))
                    .route(web::get().to(tenant::get_tenant))
                    .wrap(RequirePermission::system("tenant:update", jwt.clone(), audit.clone()).with_redis_opt(redis.clone()))
                    .route(web::patch().to(tenant::update_tenant))
                    .wrap(RequirePermission::system("tenant:delete", jwt.clone(), audit.clone()).with_redis_opt(redis))
                    .route(web::delete().to(tenant::delete_tenant))
            )
            // Verify Domain
            .route("/{id}/verify-domain", web::post().to(tenant::verify_domain))
    );
}

#[allow(dead_code)]
pub fn tenant_routes(_config: &mut web::ServiceConfig, _jwt: std::sync::Arc<crate::core::utils::jwt::JwtService>, _audit: std::sync::Arc<crate::core::infrastructure::audit::AuditService>) {
    // Legacy tenant routes if needed for business scope
}

use ntex::web;
use crate::modules::system::interface::http::handlers::audit;

pub fn audit_routes(
    config: &mut web::ServiceConfig, 
    jwt: std::sync::Arc<crate::core::utils::jwt::JwtService>, 
    audit_svc: std::sync::Arc<crate::core::infrastructure::audit::AuditService>,
    redis: Option<std::sync::Arc<crate::core::infrastructure::redis::Redis>>,
) {
    use crate::core::infrastructure::permission_middleware::RequirePermission;

    config.service(
        web::scope("/logs")
            .wrap(RequirePermission::new("system:audit:read", jwt.clone(), audit_svc.clone()).with_redis_opt(redis))
            .route("", web::get().to(audit::list_audit_logs))
    );
}

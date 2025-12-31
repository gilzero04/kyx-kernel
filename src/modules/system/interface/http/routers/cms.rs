use ntex::web;
use crate::modules::system::interface::http::handlers::cms;

// Public routes removed - handled in SystemModule directly for tail matching

pub fn admin_routes(
    config: &mut web::ServiceConfig, 
    jwt: std::sync::Arc<crate::core::utils::jwt::JwtService>, 
    audit: std::sync::Arc<crate::core::infrastructure::audit::AuditService>,
    redis: Option<std::sync::Arc<crate::core::infrastructure::redis::Redis>>,
) {
    use crate::core::infrastructure::permission_middleware::RequirePermission;

    config.service(
        web::scope("/cms")
            .wrap(RequirePermission::new("cms:read", jwt, audit).with_redis_opt(redis))
            .route("/pages", web::get().to(cms::list_admin_pages))
            .route("/pages/{id}", web::get().to(cms::get_admin_page))
            .route("/pages", web::post().to(cms::create_admin_page))
            .route("/pages/{id}", web::put().to(cms::update_admin_page))
            .route("/pages/{id}", web::delete().to(cms::delete_admin_page))
    );
}

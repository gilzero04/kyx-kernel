use ntex::web;
use crate::modules::system::interface::http::handlers::theme;
use crate::core::infrastructure::permission_middleware::RequirePermission;

pub fn theme_routes(
    cfg: &mut web::ServiceConfig,
    jwt: std::sync::Arc<crate::core::utils::jwt::JwtService>,
    audit: std::sync::Arc<crate::core::infrastructure::audit::AuditService>,
    redis: Option<std::sync::Arc<crate::core::infrastructure::redis::Redis>>,
) {
    // Public theme listing (no auth required for theme loading on login page)
    cfg.service(
        web::resource("/themes")
            .route(web::get().to(theme::list_themes))
    );

    // Protected theme management routes with granular permissions
    cfg.service(
        web::scope("/themes")
            // POST /themes/import - Import new theme
            .service(
                web::resource("/import")
                    .wrap(RequirePermission::new("theme:import", jwt.clone(), audit.clone()).with_redis_opt(redis.clone()))
                    .route(web::post().to(theme::import_theme))
            )
            // PATCH /themes/{id}/activate - Activate theme
            .service(
                web::resource("/{id}/activate")
                    .wrap(RequirePermission::new("theme:activate", jwt.clone(), audit.clone()).with_redis_opt(redis.clone()))
                    .route(web::patch().to(theme::set_active_theme))
            )
            // PATCH /themes/{id}/sharing - Update sharing status
            .service(
                web::resource("/{id}/sharing")
                    .wrap(RequirePermission::new("theme:update", jwt.clone(), audit.clone()).with_redis_opt(redis.clone()))
                    .route(web::patch().to(theme::set_sharing))
            )
            // POST /themes/{id}/update - Update theme by uploading new ZIP (SuperAdmin Owner only)
            .service(
                web::resource("/{id}/update")
                    .wrap(RequirePermission::new("theme:update", jwt.clone(), audit.clone()).with_redis_opt(redis.clone()))
                    .route(web::post().to(theme::update_theme))
            )
            // DELETE /themes/{id} - Delete theme
            .service(
                web::resource("/{id}")
                    .guard(web::guard::Delete())
                    .wrap(RequirePermission::new("theme:delete", jwt, audit).with_redis_opt(redis))
                    .route(web::delete().to(theme::delete_theme))
            )
    );
}

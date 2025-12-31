use ntex::web;
use crate::modules::system::interface::http::handlers::theme; // Correct path to handlers
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

    // Protected theme management routes
    cfg.service(
        web::scope("/themes")
            .wrap(RequirePermission::system("theme:write", jwt.clone(), audit.clone()).with_redis_opt(redis))
            
            // POST /themes/import (Multipart)
            .service(
                web::resource("/import")
                    .route(web::post().to(theme::import_theme))
            )
            
            // PATCH /themes/{id}/activate
            .route("/{id}/activate", web::patch().to(theme::set_active_theme))

            // PATCH /themes/{id}/visibility
            .route("/{id}/visibility", web::patch().to(theme::set_visibility))
            
            // DELETE /themes/{id}
            .route("/{id}", web::delete().to(theme::delete_theme))
    );
}

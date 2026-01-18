use crate::modules::media::interface::http::handlers::media;
use ntex::web;

#[allow(dead_code)]
pub fn media_routes(
    audit_service: std::sync::Arc<crate::core::infrastructure::audit::AuditService>,
) -> web::Scope<ntex::web::DefaultError> {
    web::scope("")
        .state(audit_service)
        .route("", web::post().to(media::upload_file))
        .route("/{filename}", web::get().to(media::serve_file))
}

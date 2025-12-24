use ntex::web;
use crate::modules::system::interface::http::handlers::audit;
use ntex::web::DefaultError;

pub fn audit_routes() -> web::Scope<DefaultError> {
    web::scope("/logs")
        .route("", web::get().to(audit::list_audit_logs))
}

use ntex::web;
use crate::modules::system::interface::http::handlers::workspace;
use ntex::web::DefaultError;

/// Public workspace routes
/// GET /api/v1/public/workspace/status?tenant_id=<uuid>
pub fn public_routes() -> web::Scope<DefaultError> {
    web::scope("/status")
        .route("", web::get().to(workspace::get_workspace_status))
}

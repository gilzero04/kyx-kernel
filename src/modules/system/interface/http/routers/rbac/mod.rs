use ntex::web;
use crate::modules::system::interface::http::handlers::rbac;
use ntex::web::DefaultError;

pub fn rbac_routes() -> web::Scope<DefaultError> {
    web::scope("")
        .route("/roles", web::get().to(rbac::list_roles))
        .route("/roles", web::post().to(rbac::create_role))
        .route("/roles/{id}", web::patch().to(rbac::update_role))
        .route("/roles/{id}", web::delete().to(rbac::delete_role))
        // Permissions
        .route("/permissions", web::get().to(rbac::list_permissions))
        .route("/permissions", web::post().to(rbac::create_permission))
        .route("/permissions/{id}", web::patch().to(rbac::update_permission))
        .route("/permissions/{id}", web::delete().to(rbac::delete_permission))
}

use ntex::web;
use crate::modules::system::interface::http::handlers::system;
use ntex::web::DefaultError;

pub fn public_routes() -> web::Scope<DefaultError> {
    web::scope("/status")
        .route("", web::get().to(system::get_system_status))
}

pub fn admin_routes() -> web::Scope<DefaultError> {
    web::scope("")
        .route("/settings", web::get().to(system::get_system_settings))
        .route("/test", web::get().to(system::admin_test))
}

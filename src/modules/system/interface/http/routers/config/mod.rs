use ntex::web;
use crate::modules::system::interface::http::handlers::config;
use ntex::web::DefaultError;

pub fn config_routes() -> web::Scope<DefaultError> {
    web::scope("/config")
        .route("", web::get().to(config::get_config))
        .route("", web::patch().to(config::update_config))
}

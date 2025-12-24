use ntex::web;
use crate::modules::system::interface::http::handlers::api_key;
use ntex::web::DefaultError;

pub fn api_key_routes() -> web::Scope<DefaultError> {
    web::scope("/api-keys")
        .route("", web::post().to(api_key::create_api_key))
}

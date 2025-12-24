use ntex::web;
use crate::modules::system::interface::http::handlers::cors;
use ntex::web::DefaultError;

pub fn cors_routes() -> web::Scope<DefaultError> {
    web::scope("/cors")
        .route("", web::get().to(cors::list_cors_origins))
        .route("", web::post().to(cors::add_cors_origin))
        .route("/{id}", web::delete().to(cors::delete_cors_origin))
}

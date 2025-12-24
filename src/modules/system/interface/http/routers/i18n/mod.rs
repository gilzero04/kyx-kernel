use ntex::web;
use crate::modules::system::interface::http::handlers::i18n;
use ntex::web::DefaultError;

pub fn public_routes() -> web::Scope<DefaultError> {
    web::scope("/i18n")
        .route("/locales", web::get().to(i18n::list_locales))
        .route("/{locale}", web::get().to(i18n::get_translations))
}

pub fn admin_routes() -> web::Scope<DefaultError> {
    web::scope("/i18n")
        .route("/keys", web::post().to(i18n::create_key))
        .route("/keys/{key}", web::delete().to(i18n::delete_key))
        .route("/translations", web::patch().to(i18n::update_translation))
        .route("/locales", web::post().to(i18n::create_locale))
        .route("/locales/{code}", web::delete().to(i18n::delete_locale))
}

use ntex::web;
use crate::modules::system::interface::http::handlers::user;
use ntex::web::DefaultError;

pub fn user_routes() -> web::Scope<DefaultError> {
    web::scope("/users")
        .route("", web::get().to(user::list_users))
        .route("/{id}", web::patch().to(user::update_user))
        .route("/{id}", web::delete().to(user::delete_user))
}

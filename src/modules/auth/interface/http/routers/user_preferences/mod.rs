use ntex::web;
use crate::modules::auth::interface::http::handlers::user_preferences;

/// Routes for user preferences API
pub fn user_preferences_routes(
    auth_middleware: crate::core::infrastructure::permission_middleware::RequirePermission,
) -> web::Scope<ntex::web::DefaultError> {
    web::scope("/users/me/preferences")
        .service(
            web::resource("")
                .wrap(auth_middleware)
                .route(web::get().to(user_preferences::get_my_preferences))
                .route(web::patch().to(user_preferences::update_my_preferences))
        )
}

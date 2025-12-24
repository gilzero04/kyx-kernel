use ntex::web;
use crate::modules::auth::interface::http::handlers::auth;


pub fn auth_routes(
    admin_auth: crate::core::infrastructure::permission_middleware::RequirePermission,
) -> web::Scope<ntex::web::DefaultError> {
    web::scope("/auth")
        .service(
            web::resource("/login")
                .route(web::post().to(auth::login))
        )
        .service(
            web::resource("/refresh")
                .route(web::post().to(auth::refresh_session))
        )
        .service(
            web::resource("/logout")
                .route(web::post().to(auth::logout))
        )
        .service(
            web::resource("/setup/status")
                .route(web::get().to(auth::get_setup_status))
        )
        .service(
            web::resource("/setup/verify-key")
                .route(web::post().to(auth::verify_engine_key))
        )
        .service(
            web::resource("/setup")
                .route(web::post().to(auth::initialize_system))
        )
        .service(
            web::resource("/register")
                .route(web::post().to(auth::register))
        )
        .service(
            web::resource("/users")
                .wrap(admin_auth)
                .route(web::post().to(auth::create_user))
        )
}

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
            web::resource("/setup/check-slug")
                .route(web::get().to(auth::check_slug_availability))
        )
        .service(
            web::resource("/setup")
                .route(web::post().to(auth::initialize_system))
        )
        .service(
            web::resource("/signup")
                .route(web::post().to(auth::signup))
        )
        .service(
            web::resource("/users")
                .wrap(admin_auth.clone())
                .route(web::post().to(auth::create_user))
        )
        .service(
            web::resource("/sessions")
                .route(web::get().to(auth::list_sessions))
        )
        .service(
            web::resource("/sessions/{sid}")
                .route(web::delete().to(auth::revoke_session))
        )
        // Admin Global Session Management
        .service(
            web::scope("/admin")
                .wrap(admin_auth)
                .service(
                    web::resource("/sessions")
                        .route(web::get().to(auth::admin_list_sessions))
                )
                .service(
                    web::resource("/sessions/{user_id}/{sid}")
                        .route(web::delete().to(auth::admin_revoke_session_handler))
                )
        )
}

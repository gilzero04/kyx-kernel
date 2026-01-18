use crate::core::infrastructure::permission_middleware::RequirePermission;
use crate::modules::system::interface::http::handlers::i18n;
use ntex::web;
use ntex::web::DefaultError;

pub fn public_routes() -> web::Scope<DefaultError> {
    web::scope("/i18n")
        .route("/locales", web::get().to(i18n::list_locales))
        .route("/{locale}", web::get().to(i18n::get_translations))
}

pub fn admin_routes(
    config: &mut web::ServiceConfig,
    jwt: std::sync::Arc<crate::core::utils::jwt::JwtService>,
    audit: std::sync::Arc<crate::core::infrastructure::audit::AuditService>,
    redis: Option<std::sync::Arc<crate::core::infrastructure::redis::Redis>>,
) {
    config.service(
        web::scope("/i18n")
            // GET /i18n/locales - List locales (read permission)
            .service(
                web::resource("/locales")
                    .guard(web::guard::Get())
                    .wrap(
                        RequirePermission::new("i18n:read", jwt.clone(), audit.clone())
                            .with_redis_opt(redis.clone()),
                    )
                    .route(web::get().to(i18n::list_locales)),
            )
            // POST /i18n/locales - Create locale (manage permission)
            .service(
                web::resource("/locales")
                    .guard(web::guard::Post())
                    .wrap(
                        RequirePermission::new("i18n:manage", jwt.clone(), audit.clone())
                            .with_redis_opt(redis.clone()),
                    )
                    .route(web::post().to(i18n::create_locale)),
            )
            // DELETE /i18n/locales/{code} - Delete locale (manage permission)
            .service(
                web::resource("/locales/{code}")
                    .guard(web::guard::Delete())
                    .wrap(
                        RequirePermission::new("i18n:manage", jwt.clone(), audit.clone())
                            .with_redis_opt(redis.clone()),
                    )
                    .route(web::delete().to(i18n::delete_locale)),
            )
            // GET /i18n/translations - List all translations (read permission)
            .service(
                web::resource("/translations")
                    .guard(web::guard::Get())
                    .wrap(
                        RequirePermission::new("i18n:read", jwt.clone(), audit.clone())
                            .with_redis_opt(redis.clone()),
                    )
                    .route(web::get().to(i18n::list_all_translations)),
            )
            // PATCH /i18n/translations - Update translation (manage permission)
            .service(
                web::resource("/translations")
                    .guard(web::guard::Patch())
                    .wrap(
                        RequirePermission::new("i18n:manage", jwt.clone(), audit.clone())
                            .with_redis_opt(redis.clone()),
                    )
                    .route(web::patch().to(i18n::update_translation)),
            )
            // POST /i18n/keys - Create key (manage permission)
            .service(
                web::resource("/keys")
                    .wrap(
                        RequirePermission::new("i18n:manage", jwt.clone(), audit.clone())
                            .with_redis_opt(redis.clone()),
                    )
                    .route(web::post().to(i18n::create_key)),
            )
            // DELETE /i18n/keys/{key} - Delete key (manage permission)
            .service(
                web::resource("/keys/{key}")
                    .wrap(RequirePermission::new("i18n:manage", jwt, audit).with_redis_opt(redis))
                    .route(web::delete().to(i18n::delete_key)),
            ),
    );
}

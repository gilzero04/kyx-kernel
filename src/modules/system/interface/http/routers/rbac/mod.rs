use ntex::web;
use crate::modules::system::interface::http::handlers::rbac;

pub fn rbac_routes_system(
    config: &mut web::ServiceConfig, 
    jwt: std::sync::Arc<crate::core::utils::jwt::JwtService>, 
    audit: std::sync::Arc<crate::core::infrastructure::audit::AuditService>,
    redis: Option<std::sync::Arc<crate::core::infrastructure::redis::Redis>>,
) {
    use crate::core::infrastructure::permission_middleware::RequirePermission;

    // Platform Console RBAC: Can manage Global roles & all permissions
    config.service(
        web::scope("/rbac")
            .service(
                web::scope("/roles")
                    .wrap(RequirePermission::system("role:read", jwt.clone(), audit.clone()).with_redis_opt(redis.clone()))
                    .route("", web::get().to(rbac::list_roles))
                    .service(
                        web::resource("")
                            .wrap(RequirePermission::system("role:create", jwt.clone(), audit.clone()).with_redis_opt(redis.clone()))
                            .route(web::post().to(rbac::create_role))
                    )
                    .service(
                         web::scope("/{id}")
                            .service(
                                web::resource("")
                                    .guard(web::guard::Patch())
                                    .route(web::patch().to(rbac::update_role))
                                    .wrap(RequirePermission::system("role:update", jwt.clone(), audit.clone()).with_redis_opt(redis.clone()))
                            )
                            .service(
                                web::resource("")
                                    .guard(web::guard::Delete())
                                    .route(web::delete().to(rbac::delete_role))
                                    .wrap(RequirePermission::system("role:delete", jwt.clone(), audit.clone()).with_redis_opt(redis.clone()))
                            )
                    )
            )
            .service(
                web::scope("/permissions")
                    .wrap(RequirePermission::system("permission:read", jwt.clone(), audit.clone()).with_redis_opt(redis.clone()))
                    .route("", web::get().to(rbac::list_permissions))
                    .service(
                        web::resource("")
                            .wrap(RequirePermission::system("permission:create", jwt.clone(), audit.clone()).with_redis_opt(redis.clone()))
                            .route(web::post().to(rbac::create_permission))
                    )
                    .service(
                        web::scope("/{id}")
                            .service(
                                web::resource("")
                                    .guard(web::guard::Patch())
                                    .route(web::patch().to(rbac::update_permission))
                                    .wrap(RequirePermission::system("permission:update", jwt.clone(), audit.clone()).with_redis_opt(redis.clone()))
                            )
                            .service(
                                web::resource("")
                                    .guard(web::guard::Delete())
                                    .route(web::delete().to(rbac::delete_permission))
                                    .wrap(RequirePermission::system("permission:delete", jwt.clone(), audit.clone()).with_redis_opt(redis.clone()))
                            )
                    )
            )
    );
}

pub fn rbac_routes(
    config: &mut web::ServiceConfig, 
    jwt: std::sync::Arc<crate::core::utils::jwt::JwtService>, 
    audit: std::sync::Arc<crate::core::infrastructure::audit::AuditService>,
    redis: Option<std::sync::Arc<crate::core::infrastructure::redis::Redis>>,
) {
    use crate::core::infrastructure::permission_middleware::RequirePermission;

    // Roles Management
    config.service(
        web::scope("/roles")
            .wrap(RequirePermission::new("role:read", jwt.clone(), audit.clone()).with_redis_opt(redis.clone()))
            .route("", web::get().to(rbac::list_roles))
            .service(
                web::resource("")
                    .wrap(RequirePermission::new("role:create", jwt.clone(), audit.clone()).with_redis_opt(redis.clone()))
                    .route(web::post().to(rbac::create_role))
            )
            .service(
                web::scope("/{id}")
                    .service(
                        web::resource("")
                            .guard(web::guard::Patch())
                            .route(web::patch().to(rbac::update_role))
                            .wrap(RequirePermission::new("role:update", jwt.clone(), audit.clone()).with_redis_opt(redis.clone()))
                    )
                    .service(
                        web::resource("")
                            .guard(web::guard::Delete())
                            .route(web::delete().to(rbac::delete_role))
                            .wrap(RequirePermission::new("role:delete", jwt.clone(), audit.clone()).with_redis_opt(redis.clone()))
                    )
                    .service(
                        web::resource("/permissions")
                            .guard(web::guard::Get())
                            .route(web::get().to(rbac::get_role_permissions))
                    )
                    .service(
                        web::resource("/permissions")
                            .guard(web::guard::Post())
                            .route(web::post().to(rbac::update_role_permissions))
                            .wrap(RequirePermission::new("role:update", jwt.clone(), audit.clone()).with_redis_opt(redis.clone()))
                    )
            )
    );

    // Permissions Management (added for unified /admin scope)
    config.service(
        web::scope("/permissions")
            .wrap(RequirePermission::new("permission:read", jwt.clone(), audit.clone()).with_redis_opt(redis.clone()))
            .route("", web::get().to(rbac::list_permissions))
            .service(
                web::resource("")
                    .wrap(RequirePermission::new("permission:create", jwt.clone(), audit.clone()).with_redis_opt(redis.clone()))
                    .route(web::post().to(rbac::create_permission))
            )
            .service(
                web::scope("/{id}")
                    .service(
                        web::resource("")
                            .guard(web::guard::Patch())
                            .route(web::patch().to(rbac::update_permission))
                            .wrap(RequirePermission::new("permission:update", jwt.clone(), audit.clone()).with_redis_opt(redis.clone()))
                    )
                    .service(
                        web::resource("")
                            .guard(web::guard::Delete())
                            .route(web::delete().to(rbac::delete_permission))
                            .wrap(RequirePermission::new("permission:delete", jwt.clone(), audit.clone()).with_redis_opt(redis))
                    )
            )
    );
}

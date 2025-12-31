use ntex::web;
use crate::modules::system::interface::http::handlers::user;

pub fn user_routes(
    config: &mut web::ServiceConfig, 
    jwt: std::sync::Arc<crate::core::utils::jwt::JwtService>, 
    audit: std::sync::Arc<crate::core::infrastructure::audit::AuditService>,
    redis: Option<std::sync::Arc<crate::core::infrastructure::redis::Redis>>,
) {
    use crate::core::infrastructure::permission_middleware::RequirePermission;
    
    config.service(
        web::scope("/users")
            .wrap(RequirePermission::new("user:read", jwt.clone(), audit.clone()).with_redis_opt(redis.clone()))
            .route("", web::get().to(user::list_users))
            .service(
                web::scope("/{id}")
                    .service(
                        web::resource("")
                            .guard(web::guard::Patch())
                            .route(web::patch().to(user::update_user))
                            .wrap(RequirePermission::new("user:update", jwt.clone(), audit.clone()).with_redis_opt(redis.clone()))
                    )
                    .service(
                        web::resource("")
                            .guard(web::guard::Delete())
                            .route(web::delete().to(user::delete_user))
                            .wrap(RequirePermission::new("user:delete", jwt.clone(), audit.clone()).with_redis_opt(redis.clone()))
                    )
                    .service(
                        web::resource("/reset-password")
                            .guard(web::guard::Post())
                            .route(web::post().to(user::reset_password))
                            .wrap(RequirePermission::new("user:update", jwt.clone(), audit.clone()).with_redis_opt(redis))
                    )
            )
    );
}

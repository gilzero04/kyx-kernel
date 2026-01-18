use ntex::web;
use std::sync::Arc;

use crate::core::infrastructure::audit::AuditService;
use crate::core::infrastructure::permission_middleware::RequirePermission;
use crate::core::infrastructure::redis::Redis;
use crate::core::utils::jwt::JwtService;
use crate::modules::system::interface::http::handlers::plugin;

pub fn plugin_routes(
    config: &mut web::ServiceConfig,
    jwt: Arc<JwtService>,
    audit: Arc<AuditService>,
    redis: Option<Arc<Redis>>,
) {
    config.service(
        web::scope("/plugins")
            // GET /plugins - List all plugins
            .service(
                web::resource("")
                    .guard(web::guard::Get())
                    .wrap(
                        RequirePermission::new("plugin:read", jwt.clone(), audit.clone())
                            .with_redis_opt(redis.clone()),
                    )
                    .route(web::get().to(plugin::list_plugins)),
            )
            // POST /plugins - Install new plugin
            .service(
                web::resource("")
                    .guard(web::guard::Post())
                    .wrap(
                        RequirePermission::new("plugin:install", jwt.clone(), audit.clone())
                            .with_redis_opt(redis.clone()),
                    )
                    .route(web::post().to(plugin::install_plugin)),
            )
            // POST /plugins/analyze - Analyze plugin security
            .service(
                web::resource("/analyze")
                    .wrap(
                        RequirePermission::new("plugin:install", jwt.clone(), audit.clone())
                            .with_redis_opt(redis.clone()),
                    )
                    .route(web::post().to(plugin::analyze_plugin_security)),
            )
            // POST /plugins/install-approved - Install with approval
            .service(
                web::resource("/install-approved")
                    .wrap(
                        RequirePermission::new("plugin:approve", jwt.clone(), audit.clone())
                            .with_redis_opt(redis.clone()),
                    )
                    .route(web::post().to(plugin::install_plugin_with_approval)),
            )
            // GET /plugins/{id} - Get plugin details
            .service(
                web::resource("/{id}")
                    .guard(web::guard::Get())
                    .wrap(
                        RequirePermission::new("plugin:read", jwt.clone(), audit.clone())
                            .with_redis_opt(redis.clone()),
                    )
                    .route(web::get().to(plugin::get_plugin)),
            )
            // DELETE /plugins/{id} - Uninstall plugin
            .service(
                web::resource("/{id}")
                    .guard(web::guard::Delete())
                    .wrap(
                        RequirePermission::new("plugin:uninstall", jwt.clone(), audit.clone())
                            .with_redis_opt(redis.clone()),
                    )
                    .route(web::delete().to(plugin::uninstall_plugin)),
            )
            // POST /plugins/{id}/enable - Enable plugin
            .service(
                web::resource("/{id}/enable")
                    .wrap(
                        RequirePermission::new("plugin:enable", jwt.clone(), audit.clone())
                            .with_redis_opt(redis.clone()),
                    )
                    .route(web::post().to(plugin::enable_plugin)),
            )
            // POST /plugins/{id}/disable - Disable plugin
            .service(
                web::resource("/{id}/disable")
                    .wrap(
                        RequirePermission::new("plugin:enable", jwt.clone(), audit.clone())
                            .with_redis_opt(redis.clone()),
                    )
                    .route(web::post().to(plugin::disable_plugin)),
            )
            // PUT /plugins/{id}/config - Update plugin config
            .service(
                web::resource("/{id}/config")
                    .wrap(
                        RequirePermission::new("plugin:configure", jwt.clone(), audit.clone())
                            .with_redis_opt(redis.clone()),
                    )
                    .route(web::put().to(plugin::update_plugin_config)),
            )
            // GET /plugins/{id}/security - Get security info
            .service(
                web::resource("/{id}/security")
                    .wrap(RequirePermission::new("plugin:read", jwt, audit).with_redis_opt(redis))
                    .route(web::get().to(plugin::get_plugin_security)),
            ),
    );
}

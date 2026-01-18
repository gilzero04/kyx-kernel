use crate::core::AppModule;
use crate::core::infrastructure::audit::AuditService;
use crate::core::infrastructure::config_service::ConfigService;
use crate::core::infrastructure::database::Database;
use crate::core::infrastructure::redis::Redis;
use crate::core::utils::jwt::JwtService;
use crate::modules::auth::application::services::auth::AuthService;
use ntex::web;
use std::sync::Arc;
// ApiKeyService removed from here as it is no longer used for manual setup in this module

pub mod application;
pub mod domain;
pub mod infrastructure;
pub mod interface;

pub struct AuthModule {
    pub service: Arc<AuthService>,
    db: Arc<Database>,
    redis: Arc<Redis>,
    jwt: Arc<JwtService>,
    audit: Arc<AuditService>,
    config: Arc<ConfigService>,
}

impl AuthModule {
    pub fn new(
        db: Arc<Database>,
        redis: Arc<Redis>,
        jwt: Arc<JwtService>,
        audit: Arc<AuditService>,
        config: Arc<ConfigService>,
    ) -> Self {
        let auth_service = Arc::new(AuthService::new(
            db.clone(),
            redis.clone(),
            jwt.clone(),
            audit.clone(),
            config.clone(),
        ));
        Self {
            service: auth_service,
            db,
            redis,
            jwt,
            audit,
            config,
        }
    }
}

impl AppModule for AuthModule {
    fn name(&self) -> &str {
        "auth"
    }

    fn try_configure(&self, config: &mut web::ServiceConfig) -> Result<(), crate::core::AppError> {
        let _service = self.service.clone();
        // Create Middleware for user:write permission
        let admin_auth =
            crate::core::infrastructure::permission_middleware::RequirePermission::new(
                "user:write",
                self.jwt.clone(),
                self.audit.clone(),
            )
            .with_redis(self.redis.clone());

        // Create Middleware for basic auth (any authenticated user)
        let user_auth = crate::core::infrastructure::permission_middleware::RequirePermission::new(
            "", // Empty string = just check valid JWT, no specific permission
            self.jwt.clone(),
            self.audit.clone(),
        )
        .with_redis(self.redis.clone());

        // Auth routes (login, register, etc.)
        config.service(
            interface::http::routers::auth::auth_routes(admin_auth, user_auth.clone())
                .state(self.service.clone())
                .state(self.config.clone()),
        );

        // User preferences routes (any authenticated user)
        config.service(
            interface::http::routers::user_preferences::user_preferences_routes()
                .wrap(user_auth)
                .state(self.db.clone()),
        );

        Ok(())
    }
}

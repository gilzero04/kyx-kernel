use ntex::web;
use std::sync::Arc;
use crate::core::AppModule;
use crate::core::infrastructure::database::Database;
use crate::core::infrastructure::redis::Redis;
use crate::core::utils::jwt::JwtService;
use crate::core::infrastructure::audit::AuditService;
use crate::core::infrastructure::config_service::ConfigService;
use crate::modules::auth::application::services::auth::AuthService;

pub mod domain;
pub mod application;
pub mod infrastructure;
pub mod interface;

pub struct AuthModule {
    pub service: Arc<AuthService>,
    jwt: Arc<JwtService>,
    audit: Arc<AuditService>,
}

impl AuthModule {
    pub fn new(db: Arc<Database>, redis: Arc<Redis>, jwt: Arc<JwtService>, audit: Arc<AuditService>, config: Arc<ConfigService>) -> Self {
        let auth_service = Arc::new(AuthService::new(db, redis, jwt.clone(), audit.clone(), config));
        Self { 
            service: auth_service,
            jwt,
            audit,
        }
    }
}

impl AppModule for AuthModule {
    fn name(&self) -> &str {
        "auth"
    }

    fn try_configure(&self, config: &mut web::ServiceConfig) -> Result<(), crate::core::AppError> {
        let _service = self.service.clone();
        // Create Middleware
        let admin_auth = crate::core::infrastructure::permission_middleware::RequirePermission::new(
            "user:write",
            self.jwt.clone(),
            self.audit.clone(),
        );

        config.service(
            interface::http::routers::auth::auth_routes(admin_auth)
                .state(self.service.clone())
        );
        Ok(())
    }
}

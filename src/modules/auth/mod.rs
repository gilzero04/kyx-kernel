use ntex::web;
use std::sync::Arc;
use crate::core::AppModule;
use crate::core::infrastructure::database::Database;
use crate::core::infrastructure::redis::Redis;
use crate::core::utils::jwt::JwtService;
use crate::core::infrastructure::audit::AuditService;
use crate::core::infrastructure::config_service::ConfigService;
use crate::modules::auth::application::login_service::AuthService;

pub mod domain;
pub mod application;
pub mod infrastructure;
pub mod interface;

pub struct AuthModule {
    pub service: Arc<AuthService>,
}

impl AuthModule {
    pub fn new(db: Arc<Database>, redis: Arc<Redis>, jwt: Arc<JwtService>, audit: Arc<AuditService>, config: Arc<ConfigService>) -> Self {
        let auth_service = Arc::new(AuthService::new(db, redis, jwt, audit, config));
        Self { service: auth_service }
    }
}

impl AppModule for AuthModule {
    fn name(&self) -> &str {
        "auth"
    }

    fn try_configure(&self, config: &mut web::ServiceConfig) -> Result<(), crate::core::AppError> {
        let _service = self.service.clone();
        config.service(
            web::scope("/auth")
                .state(self.service.clone())
                .service(interface::http::login)
                .service(interface::http::get_setup_status)
                .service(interface::http::verify_engine_key)
                .service(interface::http::initialize_system)
                .service(interface::http::register)
                .service(interface::http::refresh_session)
                .service(interface::http::logout)
        );
        Ok(())
    }
}

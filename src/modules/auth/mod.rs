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
            web::scope("/auth")
                .state(self.service.clone())
                .service(interface::http::login)
                .service(interface::http::get_setup_status)
                .service(interface::http::verify_engine_key)
                .service(interface::http::initialize_system)
                .service(interface::http::register)
                .service(interface::http::refresh_session)
                .service(interface::http::logout)
                .service(
                    web::resource("/users")
                        .wrap(admin_auth)
                        .route(web::post().to(interface::http::create_user))
                )
        );
        Ok(())
    }
}

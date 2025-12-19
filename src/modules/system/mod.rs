use ntex::web;
use crate::core::AppModule;
use crate::core::infrastructure::redis::Redis;
use crate::core::infrastructure::audit::AuditService;
use crate::core::infrastructure::database::Database;
use crate::core::utils::jwt::JwtService;
use crate::core::domain::auth::UserRole;
use crate::core::infrastructure::auth_middleware::RequireRole;
use crate::core::infrastructure::config_service::ConfigService;
use crate::core::infrastructure::cors::CorsManager;
use crate::modules::system::application::api_key_service::ApiKeyService;
use crate::modules::system::application::cors_service::CORSService;
use std::sync::Arc;

pub mod domain;
pub mod application;
pub mod infrastructure;
pub mod interface;

pub struct SystemModule {
    _redis: Arc<Redis>,
    db: Arc<Database>,
    jwt: Arc<JwtService>,
    audit: Arc<AuditService>,
    config: Arc<ConfigService>,
    api_key_service: Arc<ApiKeyService>,
    cors_service: Arc<CORSService>,
}

impl SystemModule {
    pub fn new(
        redis: Arc<Redis>,
        db: Arc<Database>,
        jwt: Arc<JwtService>,
        audit: Arc<AuditService>,
        config: Arc<ConfigService>,
        cors_manager: Arc<CorsManager>,
    ) -> Self {
        let api_key_service = Arc::new(ApiKeyService::new(db.clone()));
        let cors_service = Arc::new(CORSService::new(db.clone(), cors_manager));
        Self {
            _redis: redis,
            db,
            jwt,
            audit,
            config,
            api_key_service,
            cors_service,
        }
    }
}

impl AppModule for SystemModule {
    fn name(&self) -> &str {
        "system"
    }

    fn try_configure(&self, config: &mut web::ServiceConfig) -> Result<(), crate::core::AppError> {
        let config_service = self.config.clone();
        let audit_service = self.audit.clone();
        let api_key_s = self.api_key_service.clone();
        let cors_s = self.cors_service.clone();
        
        // 0. Mixed System Scope (Public + Admin)
        config.service(
            web::scope("/system")
                .state(self.db.clone())
                .state(config_service)
                .state(audit_service)
                .state(api_key_s.clone())
                .state(cors_s)
                // Public: System Status
                .service(interface::http::get_system_status)
                // Protected: Admin Routes
                .service(
                    web::scope("")
                        .wrap(RequireRole::new(UserRole::Admin, self.jwt.clone(), self.audit.clone()))
                        .service(interface::http::get_config)
                        .service(interface::http::update_config)
                        .service(interface::http::create_api_key)
                        .service(interface::http::list_cors_origins)
                        .service(interface::http::add_cors_origin)
                        .service(interface::http::admin_test)
                )
        );

        // 2. Headless/Public Scope (No JWT required, handlers check API Key)
        config.service(
            web::scope("/public/system")
                .state(api_key_s)
                .service(interface::http::get_system_info)
        );

        Ok(())
    }
}

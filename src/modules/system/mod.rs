use ntex::web;
use crate::core::AppModule;
use crate::core::infrastructure::redis::Redis;
use crate::core::infrastructure::audit::AuditService;
use crate::core::infrastructure::database::Database;
use crate::core::utils::jwt::JwtService;
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
        
        // RequirePermission middleware for admin endpoints
        let admin_auth = crate::core::infrastructure::permission_middleware::RequirePermission::new(
            "system:manage",
            self.jwt.clone(),
            self.audit.clone(),
        );

        // RequirePermission middleware for user management
        let user_auth = crate::core::infrastructure::permission_middleware::RequirePermission::new(
            "user:write",
            self.jwt.clone(),
            self.audit.clone(),
        );
        
        // 1. Public System Scope (no auth required)
        config.service(
            web::scope("/system")
                .state(self.db.clone())
                .state(config_service.clone())
                .state(audit_service.clone())
                // Public: System Status (no auth) - for app startup/branding
                .service(interface::http::get_system_status)
        );
        
        // 2. User Management Scope (user:write permission)
        // Must be registered BEFORE /admin to matching priority
        config.service(
            web::scope("/admin/users")
                .wrap(user_auth)
                .state(self.db.clone())
                .state(audit_service.clone())
                .service(interface::http::users_handler::list_users)
                .service(interface::http::users_handler::update_user)
                .service(interface::http::users_handler::delete_user)
        );

        // 3. Protected Admin Scope (Requires system:manage permission)
        // Using /admin instead of /system/admin to avoid scope conflict
        config.service(
            web::scope("/admin")
                .wrap(admin_auth)
                .state(self.db.clone())
                .state(config_service)
                .state(audit_service.clone())
                .state(api_key_s.clone())
                .state(cors_s)
                // All admin endpoints require system:manage permission
                .service(interface::http::get_config)
                .service(interface::http::update_config)
                .service(interface::http::create_api_key)
                .service(interface::http::list_cors_origins)
                .service(interface::http::add_cors_origin)
                .service(interface::http::get_system_settings)
                .service(interface::http::list_audit_logs)
                .service(interface::http::list_tenants)
                .service(interface::http::update_owner)
                .service(interface::http::list_permissions)
                .service(interface::http::create_permission)
                .service(interface::http::update_permission)
                .service(interface::http::delete_permission)
                .service(interface::http::list_roles)
                .service(interface::http::create_role)
                .service(interface::http::update_role)
                .service(interface::http::delete_role)
                .service(interface::http::admin_test)
        );


        // 4. Headless/Public Scope (No JWT required, handlers check API Key)
        config.service(
            web::scope("/public/system")
                .state(api_key_s)
                .service(interface::http::get_system_info)
        );

        Ok(())
    }
}

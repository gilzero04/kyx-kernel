use ntex::web;
use crate::core::AppModule;
use crate::core::infrastructure::redis::Redis;
use crate::core::infrastructure::audit::AuditService;
use crate::core::infrastructure::database::Database;
use crate::core::utils::jwt::JwtService;
use crate::core::infrastructure::config_service::ConfigService;
use crate::core::infrastructure::cors::CorsManager;
use crate::modules::system::application::services::api_key::ApiKeyService;
use crate::modules::system::application::services::i18n::I18nService;
use crate::modules::system::infrastructure::repositories::i18n::PostgresI18nRepositoryImpl;
use crate::core::infrastructure::ai_service::AIService;
use crate::modules::system::application::services::cors::CORSService;
use crate::modules::system::application::services::audit::AuditQueryService;
use crate::modules::system::infrastructure::repositories::audit::PostgresAuditRepository;
use crate::modules::system::infrastructure::repositories::api_key::PostgresApiKeyRepository;
use crate::modules::system::infrastructure::repositories::cors::PostgresCorsRepository;
use crate::modules::system::infrastructure::repositories::tenant::PostgresTenantRepository;
use crate::modules::system::application::services::tenant::TenantService;
use crate::modules::system::infrastructure::repositories::rbac::PostgresRbacRepository;
use crate::modules::system::application::services::rbac::RbacService;
use crate::modules::system::infrastructure::repositories::user::PostgresUserRepository;
use crate::modules::system::application::services::user::UserAdminService;
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
    i18n_service: Arc<I18nService>,
    audit_query_service: Arc<AuditQueryService>,
    tenant_service: Arc<TenantService>,
    rbac_service: Arc<RbacService>,
    user_service: Arc<UserAdminService>,
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
        let api_key_repo = Arc::new(PostgresApiKeyRepository::new(db.clone()));
        let api_key_service = Arc::new(ApiKeyService::new(api_key_repo));

        let cors_repo = Arc::new(PostgresCorsRepository::new(db.clone()));
        let cors_service = Arc::new(CORSService::new(cors_repo, cors_manager));
        
        // Tenant
        let tenant_repo = Arc::new(PostgresTenantRepository::new(db.clone()));
        let tenant_service = Arc::new(TenantService::new(tenant_repo));

        // RBAC
        let rbac_repo = Arc::new(PostgresRbacRepository::new(db.clone()));
        let rbac_service = Arc::new(RbacService::new(rbac_repo));

        // User
        let user_repo = Arc::new(PostgresUserRepository::new(db.clone()));
        let user_service = Arc::new(UserAdminService::new(user_repo));

        // I18n
        let i18n_repo = Arc::new(PostgresI18nRepositoryImpl::new(db.clone()));
        let ai_service = Arc::new(AIService::new(config.clone()));
        let i18n_service = Arc::new(I18nService::new(i18n_repo, audit.clone(), ai_service));

        // Audit Query (Read)
        let audit_repo = Arc::new(PostgresAuditRepository::new(db.clone()));
        let audit_query_service = Arc::new(AuditQueryService::new(audit_repo));

        Self {
            _redis: redis,
            db,
            jwt,
            audit,
            config,
            api_key_service,
            cors_service,
            i18n_service,
            audit_query_service,
            tenant_service,
            rbac_service,
            user_service,
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
        let i18n_s = self.i18n_service.clone();
        let audit_query_s = self.audit_query_service.clone();
        let tenant_s = self.tenant_service.clone();
        
        // RequirePermission middleware for admin endpoints
        let admin_auth = crate::core::infrastructure::permission_middleware::RequirePermission::new(
            "system:manage",
            self.jwt.clone(),
            self.audit.clone(),
        );
        
        // 1. Public System Scope (no auth required)
        config.service(
            web::scope("/system")
                .state(self.db.clone())
                .state(config_service.clone())
                .state(audit_service.clone())
                .state(i18n_s.clone()) 
                // Public: System Status (no auth) - for app startup/branding
                .service(interface::http::routers::system::public_routes())
                .service(interface::http::routers::i18n::public_routes())
        );
        


        // 3. Consolidated Protected Admin Scope (Requires system:manage permission)
        config.service(
            web::scope("/admin")
                .wrap(admin_auth.clone())
                .state(self.db.clone())
                .state(self.jwt.clone()) // Required for delete_user self-check
                .state(config_service)
                .state(audit_service.clone()) // Write (Core)
                .state(audit_query_s) // Read (System Module)
                .state(api_key_s.clone())
                .state(cors_s)
                .state(i18n_s.clone())
                .state(tenant_s)
                .state(self.rbac_service.clone())
                .state(self.user_service.clone())
                .state(self.i18n_service.clone())
                // Sub-Routers with specific prefixes
                .service(interface::http::routers::audit::audit_routes()) 
                .service(interface::http::routers::config::config_routes())
                .service(interface::http::routers::api_key::api_key_routes())
                .service(interface::http::routers::cors::cors_routes())
                .service(interface::http::routers::tenant::tenant_routes())
                .service(interface::http::routers::user::user_routes())
                .service(interface::http::routers::i18n::admin_routes())
                
                // System Settings & Status (Direct)
                .route("/settings", web::get().to(interface::http::handlers::system::get_system_settings))
                .route("/test", web::get().to(interface::http::handlers::system::admin_test))

                // RBAC (Explicit Sub-Scopes)
                .service(
                    web::scope("/roles")
                        .route("", web::get().to(interface::http::handlers::rbac::list_roles))
                        .route("", web::post().to(interface::http::handlers::rbac::create_role))
                        .route("/{id}", web::patch().to(interface::http::handlers::rbac::update_role))
                        .route("/{id}", web::delete().to(interface::http::handlers::rbac::delete_role))
                )
                .service(
                    web::scope("/permissions")
                        .route("", web::get().to(interface::http::handlers::rbac::list_permissions))
                        .route("", web::post().to(interface::http::handlers::rbac::create_permission))
                        .route("/{id}", web::patch().to(interface::http::handlers::rbac::update_permission))
                        .route("/{id}", web::delete().to(interface::http::handlers::rbac::delete_permission))
                )
        );
        
        // 4. Headless/Public Scope
        config.service(
            web::scope("/public/system")
                .state(api_key_s)
                .route("/info", web::get().to(interface::http::handlers::system::get_system_info))
        );

        Ok(())
    }
}

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
use crate::modules::system::infrastructure::repositories::cors::PostgresCorsRepository;
use crate::modules::system::infrastructure::repositories::tenant::PostgresTenantRepository;
use crate::modules::system::application::services::tenant::TenantService;
use crate::modules::system::application::services::tenant::domain_verification::DomainVerificationService;
use crate::modules::system::infrastructure::repositories::rbac::PostgresRbacRepository;
use crate::modules::system::application::services::rbac::RbacService;
use crate::modules::system::infrastructure::repositories::user::PostgresUserRepository;
use crate::modules::system::application::services::user::UserAdminService;
use crate::modules::system::infrastructure::repositories::theme::PostgresThemeRepository;
use crate::modules::system::application::services::theme::ThemeService;
use crate::modules::system::infrastructure::repositories::cms::PostgresCmsRepository;
use crate::modules::system::application::services::cms::CmsService;
use crate::modules::system::domain::plugin::registry::PluginRegistry;
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
    tenant_service: Arc<TenantService>,
    domain_verification_service: Arc<DomainVerificationService>,
    rbac_service: Arc<RbacService>,
    user_service: Arc<UserAdminService>,
    theme_service: Arc<ThemeService>,
    audit_query_service: Arc<AuditQueryService>,
    cms_service: Arc<CmsService>,
    plugin_registry: Arc<PluginRegistry>,
}

impl SystemModule {
    pub fn new(
        redis: Arc<Redis>,
        db: Arc<Database>,
        jwt: Arc<JwtService>,
        audit: Arc<AuditService>,
        config: Arc<ConfigService>,
        cors_manager: Arc<CorsManager>,
        api_key_service: Arc<ApiKeyService>,
    ) -> Self {

        let cors_repo = Arc::new(PostgresCorsRepository::new(db.clone()));
        let cors_service = Arc::new(CORSService::new(cors_repo, cors_manager));
        
        // Tenant
        let tenant_repo = Arc::new(PostgresTenantRepository::new(db.clone()));
        let tenant_service = Arc::new(TenantService::new(tenant_repo.clone()));
        let domain_verification_service = Arc::new(DomainVerificationService::new(tenant_repo.clone()));

        // RBAC
        let rbac_repo = Arc::new(PostgresRbacRepository::new(db.clone()));
        let rbac_service = Arc::new(RbacService::new(rbac_repo));

        // User
        let user_repo = Arc::new(PostgresUserRepository::new(db.clone()));
        let user_service = Arc::new(UserAdminService::new(user_repo));

        // Theme
        let theme_repo = Arc::new(PostgresThemeRepository::new(db.clone()));
        let theme_service = Arc::new(ThemeService::new(theme_repo));

        // I18n
        let i18n_repo = Arc::new(PostgresI18nRepositoryImpl::new(db.clone()));
        let ai_service = Arc::new(AIService::new(config.clone()));
        let i18n_service = Arc::new(I18nService::new(i18n_repo, audit.clone(), ai_service));

        // Audit Query (Read)
        let audit_repo = Arc::new(PostgresAuditRepository::new(db.clone()));
        let audit_query_service = Arc::new(AuditQueryService::new(audit_repo));

        // CMS
        let cms_repo = Arc::new(PostgresCmsRepository::new(db.clone()));
        let cms_service = Arc::new(CmsService::new(cms_repo));

        // Plugin Registry
        let plugin_registry = Arc::new(PluginRegistry::new(db.clone(), redis.clone()));

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
            domain_verification_service,
            rbac_service,
            user_service,
            theme_service,
            cms_service,
            plugin_registry,
        }
    }

    pub async fn seed_themes(&self) -> crate::core::AppResult<()> {
        self.theme_service.seed_default_themes().await
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
        let domain_verification_s = self.domain_verification_service.clone();
        let theme_s = self.theme_service.clone();
        
// DELETED: Global admin_auth wrapper replaced by granular guards in routers

        
        // 1. System Scope (Public & Protected Platform Management)
        let cms_s = self.cms_service.clone();
        config.service(
            web::scope("/system")
                .state(self.db.clone())
                .state(config_service.clone())
                .state(audit_service.clone())
                .state(i18n_s.clone()) 
                .state(self.jwt.clone()) 
                .state(audit_query_s.clone()) 
                .state(api_key_s.clone())
                .state(cors_s.clone())
                .state(tenant_s.clone())
                .state(domain_verification_s.clone())
                .state(self.rbac_service.clone())
                .state(theme_s.clone())
                // Public Routes
                .service(interface::http::routers::system::public_routes())
                .service(interface::http::routers::i18n::public_routes())
                // Protected Platform Routes (Each router is now self-protected)
                .configure(|conf| interface::http::routers::tenant::tenant_routes_system(conf, self.jwt.clone(), self.audit.clone(), Some(self._redis.clone())))
                .configure(|conf| interface::http::routers::rbac::rbac_routes_system(conf, self.jwt.clone(), self.audit.clone(), Some(self._redis.clone())))
                .configure(|conf| interface::http::routers::audit::audit_routes(conf, self.jwt.clone(), self.audit.clone(), Some(self._redis.clone())))
                .configure(|conf| interface::http::routers::config::config_routes(conf, self.jwt.clone(), self.audit.clone(), Some(self._redis.clone())))
                .configure(|conf| interface::http::routers::api_key::api_key_routes(conf, self.jwt.clone(), self.audit.clone(), Some(self._redis.clone())))
                .configure(|conf| interface::http::routers::theme::theme_routes(conf, self.jwt.clone(), self.audit.clone(), Some(self._redis.clone())))
                .configure(|conf| interface::http::routers::cors::cors_routes(conf, self.jwt.clone(), self.audit.clone(), Some(self._redis.clone())))
                .configure(|conf| interface::http::routers::i18n::admin_routes(conf, self.jwt.clone(), self.audit.clone(), Some(self._redis.clone())))
                .configure(|conf| interface::http::routers::system::admin_routes(conf, self.jwt.clone(), self.audit.clone(), Some(self._redis.clone())))
        );

        // 3. Tenant Admin Scope (Business Management)
        config.service(
            web::scope("/admin")
                .state(self.db.clone())
                .state(self.jwt.clone()) 
                .state(config_service.clone())
                .state(audit_service.clone()) 
                .state(self.rbac_service.clone())
                .state(self.user_service.clone())
                .state(tenant_s.clone())
                .state(cors_s.clone())
                .state(audit_query_s)
                .state(i18n_s.clone())
                .state(theme_s.clone())
                .state(cms_s.clone())
                .state(self.plugin_registry.clone())
                // Business level operations
                .configure(|conf| interface::http::routers::user::user_routes(conf, self.jwt.clone(), self.audit.clone(), Some(self._redis.clone())))
                .configure(|conf| interface::http::routers::rbac::rbac_routes(conf, self.jwt.clone(), self.audit.clone(), Some(self._redis.clone())))
                .configure(|conf| interface::http::routers::tenant::tenant_routes_system(conf, self.jwt.clone(), self.audit.clone(), Some(self._redis.clone())))
                .configure(|conf| interface::http::routers::config::config_routes(conf, self.jwt.clone(), self.audit.clone(), Some(self._redis.clone())))
                .configure(|conf| interface::http::routers::cors::cors_routes(conf, self.jwt.clone(), self.audit.clone(), Some(self._redis.clone())))
                .configure(|conf| interface::http::routers::audit::audit_routes(conf, self.jwt.clone(), self.audit.clone(), Some(self._redis.clone())))
                .configure(|conf| interface::http::routers::i18n::admin_routes(conf, self.jwt.clone(), self.audit.clone(), Some(self._redis.clone())))
                .configure(|conf| interface::http::routers::theme::theme_routes(conf, self.jwt.clone(), self.audit.clone(), Some(self._redis.clone())))
                .configure(|conf| interface::http::routers::system::admin_routes(conf, self.jwt.clone(), self.audit.clone(), Some(self._redis.clone())))
                .configure(|conf| interface::http::routers::cms::admin_routes(conf, self.jwt.clone(), self.audit.clone(), Some(self._redis.clone())))
                // Plugin Management (wired via handler.configure)
                .configure(interface::http::handlers::plugin::configure)
        );
        
        // 4. Headless/Public Scope
        config.service(
            web::scope("/public/system")
                .state(api_key_s)
                .state(self.cms_service.clone())
                .state(self.tenant_service.clone())
                .route("/info", web::get().to(interface::http::handlers::system::get_system_info))
                .route("/tenants/{slug}", web::get().to(interface::http::handlers::tenant::get_tenant_by_slug))
                .service(
                    web::scope("/cms/pages/{tenant_id}")
                        .default_service(web::get().to(interface::http::handlers::cms::get_page_by_slug))
                )
        );

        Ok(())
    }
}

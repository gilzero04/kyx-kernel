use crate::core::infrastructure::audit::AuditService;
use crate::core::infrastructure::config_service::ConfigService;
use crate::core::infrastructure::cors::CorsManager;
use crate::core::infrastructure::database::Database;
use crate::core::infrastructure::redis::Redis;
use crate::core::utils::jwt::JwtService;
use crate::modules::auth::AuthModule;
use crate::modules::media::MediaModule;
use crate::modules::system::SystemModule;
use crate::modules::system::application::services::api_key::ApiKeyService;
use crate::modules::system::infrastructure::repositories::api_key::PostgresApiKeyRepository;
use std::io;
use std::sync::Arc;

pub struct Registry {
    pub audit_service: Arc<AuditService>,
    pub config_service: Arc<ConfigService>,
    pub cors_manager: Arc<CorsManager>,
    pub jwt_service: Arc<JwtService>,
    pub api_key_service: Arc<ApiKeyService>,
    pub auth_module: Arc<AuthModule>,
    pub system_module: Arc<SystemModule>,
    pub media_module: Arc<MediaModule>,
}

impl Registry {
    pub async fn new(
        database: Arc<Database>,
        redis: Arc<Redis>,
        jwt_secret: &str,
        _engine_secret: &str,
    ) -> io::Result<Self> {
        // 1. Base Shared Services
        let audit_service = Arc::new(AuditService::new(database.clone()));
        let config_service = Arc::new(ConfigService::new(database.clone(), redis.clone()));
        let cors_manager = Arc::new(CorsManager::new(redis.clone(), database.clone()));
        let jwt_service = Arc::new(JwtService::new(jwt_secret));

        // Initial CORS refresh
        cors_manager
            .refresh()
            .await
            .map_err(|e| io::Error::other(e.message))?;

        // 2. Repositories & Domain Services
        let api_key_repo = Arc::new(PostgresApiKeyRepository::new(database.clone()));
        let api_key_service = Arc::new(ApiKeyService::new(api_key_repo));

        // 3. Modules
        let auth_module = Arc::new(AuthModule::new(
            database.clone(),
            redis.clone(),
            jwt_service.clone(),
            audit_service.clone(),
            config_service.clone(),
        ));

        let system_module = Arc::new(SystemModule::new(
            redis.clone(),
            database.clone(),
            jwt_service.clone(),
            audit_service.clone(),
            config_service.clone(),
            cors_manager.clone(),
            api_key_service.clone(),
        ));

        let media_module = Arc::new(MediaModule::new(
            database.clone(),
            audit_service.clone(),
            jwt_service.clone(),
        ));

        // 4. Background Loops / Post-init
        Self::spawn_background_tasks(cors_manager.clone(), system_module.clone());

        // Audit Log Startup
        audit_service
            .log("SYSTEM", "KERNEL_STARTUP", None, "SUCCESS", None)
            .await
            .map_err(|e| io::Error::other(e.message))?;

        Ok(Self {
            audit_service,
            config_service,
            cors_manager,
            jwt_service,
            api_key_service,
            auth_module,
            system_module,
            media_module,
        })
    }

    fn spawn_background_tasks(cors: Arc<CorsManager>, system: Arc<SystemModule>) {
        // CORS Refresh Loop
        tokio::spawn(async move {
            let mut interval = tokio::time::interval(std::time::Duration::from_secs(60));
            loop {
                interval.tick().await;
                if let Err(e) = cors.refresh().await {
                    log::error!("Failed to refresh CORS: {}", e);
                }
            }
        });

        // Theme Seeding
        tokio::spawn(async move {
            if let Err(e) = system.seed_themes().await {
                log::error!("Failed to seed default themes: {}", e);
            }
        });
    }
}

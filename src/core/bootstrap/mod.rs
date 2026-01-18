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
use std::io;
use std::sync::Arc;

pub mod infrastructure;
pub mod registry;

/// The Kernel struct holds all shared application state.
pub struct Kernel {
    pub port: u16,
    pub env: String,
    pub engine_secret: String,
    pub redis: Arc<Redis>,
    pub database: Arc<Database>,
    #[allow(dead_code)] // Kept for potential external service access
    pub audit_service: Arc<AuditService>,
    pub config_service: Arc<ConfigService>,
    pub cors_manager: Arc<CorsManager>,
    #[allow(dead_code)] // Kept for identity validation in testing or expansion
    pub jwt_service: Arc<JwtService>,
    #[allow(dead_code)] // Kept for API key management extension
    pub api_key_service: Arc<ApiKeyService>,

    // Modules
    pub auth_module: Arc<AuthModule>,
    pub system_module: Arc<SystemModule>,
    pub media_module: Arc<MediaModule>,
}

impl Kernel {
    /// Bootstraps the entire kernel.
    pub async fn init() -> io::Result<Self> {
        // 1. Load Environment & Safety Checks
        let (port, env, jwt_secret, engine_secret) = infrastructure::load_env_and_check()?;

        // 2. Initialize Infrastructure (Storage/Bus)
        let redis = infrastructure::init_redis().await?;
        let database = infrastructure::init_database().await?;

        // 3. Initialize Shared Services & Modules (Wiring)
        let registry =
            registry::Registry::new(database.clone(), redis.clone(), &jwt_secret, &engine_secret)
                .await?;

        log::info!("🚀 Kernel bootstrap completed successfully.");

        Ok(Self {
            port,
            env,
            engine_secret,
            redis,
            database,
            audit_service: registry.audit_service,
            config_service: registry.config_service,
            cors_manager: registry.cors_manager,
            jwt_service: registry.jwt_service,
            api_key_service: registry.api_key_service,
            auth_module: registry.auth_module,
            system_module: registry.system_module,
            media_module: registry.media_module,
        })
    }
}

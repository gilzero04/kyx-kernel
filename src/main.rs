use ntex::web;
use std::io;
use std::sync::Arc;
use crate::core::AppModule;
mod core;
use crate::core::infrastructure::redis::Redis;
use crate::core::infrastructure::config_service::ConfigService;
use crate::core::infrastructure::cors::CorsManager;

mod modules;
mod interface;
use utoipa::OpenApi;

#[ntex::main]
async fn main() -> io::Result<()> {
    // 1. Load Environment Variables
    dotenvy::dotenv().ok();
    env_logger::init_from_env(env_logger::Env::default().default_filter_or("info"));

    let port = std::env::var("PORT")
        .unwrap_or_else(|_| "8080".to_string())
        .parse::<u16>()
        .unwrap_or(8080);
    
    let env = std::env::var("ENVIRONMENT").unwrap_or_else(|_| "local".to_string());
    let jwt_secret = std::env::var("JWT_SECRET").expect("JWT_SECRET must be set");
    let engine_secret = std::env::var("ENGINE_SECRET_KEY").expect("ENGINE_SECRET_KEY must be set");

    // 2. Production Safety Checks
    if env == "production" {
        if jwt_secret == "change_me_immediately_in_production" || jwt_secret.len() < 32 {
            log::error!("❌ FATAL: Weak or default JWT_SECRET detected in PRODUCTION!");
            return Err(io::Error::new(io::ErrorKind::Other, "Insecure configuration in production"));
        }
        log::info!("🛡️  Production safety checks passed.");
    }

    // 3. Initialize Secure Infrastructure from Env
    let redis_url = std::env::var("REDIS_URL").expect("REDIS_URL must be set");
    let redis_pass = std::env::var("REDIS_PASSWORD").ok();
    
    let redis = Arc::new(Redis::new(&redis_url, redis_pass.as_deref())
        .await
        .map_err(|e| io::Error::new(io::ErrorKind::Other, e))?);
    
    // 4. Initialize SQL Database (PostgreSQL)
    let database_url = std::env::var("DATABASE_URL").expect("DATABASE_URL must be set");
    let database = Arc::new(crate::core::infrastructure::database::Database::new(&database_url)
        .await
        .map_err(|e| io::Error::new(io::ErrorKind::Other, e))?);
    
    // Initialize Database Tables (Migrations)
    database.initialize_tables().await.map_err(|e| io::Error::new(io::ErrorKind::Other, e))?;

    // 5. Initialize Core Services
    let audit_service = Arc::new(crate::core::infrastructure::audit::AuditService::new(database.clone()));
    let config_service = Arc::new(ConfigService::new(database.clone(), redis.clone()));
    let cors_manager = Arc::new(CorsManager::new(redis.clone(), database.clone()));
    
    // Initial CORS refresh from DB
    cors_manager.refresh().await.map_err(|e| io::Error::new(io::ErrorKind::Other, e.message))?;
    
    // Log Kernel Startup
    audit_service.log("SYSTEM", "KERNEL_STARTUP", None, "SUCCESS", None).await
        .map_err(|e| io::Error::new(io::ErrorKind::Other, e.message))?;

    // 6. Initialize Security & Modules
    let jwt_service = Arc::new(crate::core::utils::jwt::JwtService::new(&jwt_secret));
    
    // Background CORS Refresh Loop
    let cors_clone = cors_manager.clone();
    tokio::spawn(async move {
        let mut interval = tokio::time::interval(std::time::Duration::from_secs(60));
        loop {
            interval.tick().await;
            if let Err(e) = cors_clone.refresh().await {
                log::error!("Failed to refresh CORS: {}", e);
            }
        }
    });

    let api_key_repo = Arc::new(crate::modules::system::infrastructure::repositories::api_key::PostgresApiKeyRepository::new(database.clone()));
    let api_key_service = Arc::new(crate::modules::system::application::services::api_key::ApiKeyService::new(api_key_repo));

    let auth_module = Arc::new(modules::auth::AuthModule::new(
        database.clone(),
        redis.clone(), 
        jwt_service.clone(), 
        audit_service.clone(), 
        config_service.clone(),
    ));
    
    let system_module = Arc::new(modules::system::SystemModule::new(
        redis.clone(),
        database.clone(),
        jwt_service.clone(),
        audit_service.clone(),
        config_service.clone(),
        cors_manager.clone(),
        api_key_service.clone()
    ));

    let media_module = Arc::new(modules::media::MediaModule::new(
        database.clone(),
        audit_service.clone(),
        jwt_service.clone()
    ));

    println!("🚀 Kyx Kernel v{} ({}) starting on port {}...", 
        std::env::var("APP_VERSION").unwrap_or_else(|_| "0.1.0".to_string()),
        env,
        port
    );
    
    let auth = auth_module.clone();
    let system = system_module.clone();
    let media = media_module.clone();
    let config_for_rate = config_service.clone();

    let system_for_seed = system_module.clone();
    tokio::spawn(async move {
        if let Err(e) = system_for_seed.seed_themes().await {
            log::error!("Failed to seed default themes: {}", e);
        }
    });

    web::server(move || {
        let auth_m = auth.clone();
        let system_m = system.clone();
        let media_m = media.clone();
        
        web::App::new()
            .state(redis.clone())
            .wrap(web::middleware::Logger::default())
            .wrap(crate::core::infrastructure::cors_middleware::DynamicCors::new(cors_manager.clone()))
            .wrap(crate::core::infrastructure::rate_limit::DynamicRateLimit::new(config_for_rate.clone()))
            .service(
                web::resource("/api-doc/openapi.json")
                    .to(|| async {
                        web::HttpResponse::Ok()
                            .content_type("application/json")
                            .json(&interface::http::openapi::ApiDoc::openapi())
                    })
            )
            .service(
                web::resource("/swagger-ui/")
                    .to(|| async {
                        web::HttpResponse::Ok()
                            .content_type("text/html")
                            .body(
                                r#"<!DOCTYPE html>
<html lang="en">
<head>
  <meta charset="utf-8" />
  <meta name="viewport" content="width=device-width, initial-scale=1" />
  <meta name="description" content="SwaggerUI" />
  <title>Kyx Kernel - API Documentation</title>
  <link rel="stylesheet" href="https://unpkg.com/swagger-ui-dist@5/swagger-ui.css" />
</head>
<body>
<div id="swagger-ui"></div>
<script src="https://unpkg.com/swagger-ui-dist@5/swagger-ui-bundle.js" crossorigin></script>
<script>
  window.onload = () => {
    window.ui = SwaggerUIBundle({
      url: '/api-doc/openapi.json',
      dom_id: '#swagger-ui',
      deepLinking: true,
      presets: [
        SwaggerUIBundle.presets.apis
      ],
    });
  };
</script>
</body>
</html>"#
                            )
                    })
            )
            .service(
                web::scope("/api/v1")
                    .configure(move |cfg| { let _ = auth_m.try_configure(cfg); })
                    .configure(move |cfg| { let _ = system_m.try_configure(cfg); })
                    .configure(move |cfg| { let _ = media_m.try_configure(cfg); })
            )
            .service(
                web::scope("/internal")
                    .wrap(crate::core::infrastructure::handshake_middleware::EngineHandshake::new(engine_secret.clone()))
                    .service(web::resource("/status").to(|| async { 
                        web::HttpResponse::Ok().json(&serde_json::json!({"status": "ready"})) 
                     }))
            )
            // Health check endpoint (no auth required)
            .service(web::resource("/health").to(|| async {
                web::HttpResponse::Ok().json(&serde_json::json!({
                    "status": "healthy",
                    "version": std::env::var("APP_VERSION").unwrap_or_else(|_| "1.0.0".to_string()),
                    "timestamp": chrono::Utc::now().to_rfc3339()
                }))
            }))
            // Serve static theme assets (preview images, logos)
            .service(
                ntex_files::Files::new("/themes", "./assets/themes")
                    .show_files_listing()
                    .use_last_modified(true)
            )
            // Favicon handler (prevent 404 logs)
            .service(web::resource("/favicon.ico").to(|| async {
                web::HttpResponse::NoContent().finish()
            }))
    })
    .bind(("0.0.0.0", port))?
    .run()
    .await
}

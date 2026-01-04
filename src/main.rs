use ntex::web;
use std::io;
use utoipa::OpenApi;
use crate::core::AppModule;
use crate::core::bootstrap::Kernel;

mod core;
mod modules;
mod interface;

#[ntex::main]
async fn main() -> io::Result<()> {
    // 1. Initialize Kernel (Env, Infrastructure, Services, Modules)
    let kernel = Kernel::init().await?;

    println!("🚀 Kyx Kernel v{} ({}) starting on port {}...", 
        std::env::var("APP_VERSION").unwrap_or_else(|_| "0.1.0".to_string()),
        kernel.env,
        kernel.port
    );

    // 2. Start Web Server
    web::server(move || {
        let redis = kernel.redis.clone();
        let database = kernel.database.clone();
        let config_service = kernel.config_service.clone();
        let cors_manager = kernel.cors_manager.clone();
        let engine_secret = kernel.engine_secret.clone();
        
        let auth_m = kernel.auth_module.clone();
        let system_m = kernel.system_module.clone();
        let media_m = kernel.media_module.clone();

        web::App::new()
            .state(redis)
            .state(database)
            .wrap(web::middleware::Logger::default())
            .wrap(crate::core::infrastructure::cors_middleware::DynamicCors::new(cors_manager.clone()))
            .wrap(crate::core::infrastructure::rate_limit::DynamicRateLimit::new(config_service.clone()))
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
      presets: [SwaggerUIBundle.presets.apis],
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
                    .wrap(crate::core::infrastructure::handshake_middleware::EngineHandshake::new(engine_secret))
                    .service(web::resource("/status").to(|| async { 
                        web::HttpResponse::Ok().json(&serde_json::json!({"status": "ready"})) 
                     }))
            )
            .service(web::resource("/health").to(|| async {
                web::HttpResponse::Ok().json(&serde_json::json!({
                    "status": "healthy",
                    "version": std::env::var("APP_VERSION").unwrap_or_else(|_| "1.0.0".to_string()),
                    "timestamp": chrono::Utc::now().to_rfc3339()
                }))
            }))
            .service(
                ntex_files::Files::new("/themes", "./assets/themes")
                    .show_files_listing()
                    .use_last_modified(true)
            )
            .service(web::resource("/favicon.ico").to(|| async {
                web::HttpResponse::NoContent().finish()
            }))
    })
    .bind(("0.0.0.0", kernel.port))?
    .run()
    .await
}

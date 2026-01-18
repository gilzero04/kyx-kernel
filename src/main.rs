use crate::core::AppModule;
use crate::core::bootstrap::Kernel;
use base64::Engine;
use ntex::web;
use std::io;
use utoipa::OpenApi;

mod core;
mod interface;
mod modules;

/// Serve Swagger UI HTML
fn serve_swagger_ui() -> web::HttpResponse {
    web::HttpResponse::Ok().content_type("text/html").body(
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
</html>"#,
    )
}

/// Serve Redoc HTML
fn serve_redoc_ui() -> web::HttpResponse {
    web::HttpResponse::Ok()
        .content_type("text/html")
        .body(r#"<!DOCTYPE html>
<html lang="en">
<head>
  <meta charset="utf-8" />
  <meta name="viewport" content="width=device-width, initial-scale=1" />
  <title>Kyx Kernel - API Reference</title>
  <link href="https://fonts.googleapis.com/css?family=Montserrat:300,400,700|Roboto:300,400,700" rel="stylesheet">
  <style>body { margin: 0; padding: 0; }</style>
</head>
<body>
  <redoc spec-url='/api-doc/openapi.json'></redoc>
  <script src="https://cdn.redoc.ly/redoc/latest/bundles/redoc.standalone.js"></script>
</body>
</html>"#)
}

/// Check docs authentication based on ENVIRONMENT
/// - local/development: No auth required (open access)
/// - staging/production: Basic auth required (DOCS_USER/DOCS_PASSWORD)
fn check_docs_auth(req: &web::HttpRequest) -> bool {
    // Check environment mode
    let env = std::env::var("ENVIRONMENT").unwrap_or_else(|_| "local".to_string());
    let is_protected = matches!(env.as_str(), "staging" | "production" | "prod");

    if !is_protected {
        return true; // Dev mode - no auth required
    }

    // Protected mode - check Basic Auth credentials
    let docs_user = std::env::var("DOCS_USER").unwrap_or_default();
    let docs_pass = std::env::var("DOCS_PASSWORD").unwrap_or_default();

    if docs_user.is_empty() || docs_pass.is_empty() {
        return true; // No credentials set = allow access (avoid lockout)
    }

    if let Some(auth) = req.headers().get("Authorization") {
        if let Ok(auth_str) = auth.to_str() {
            if auth_str.starts_with("Basic ") {
                let encoded = &auth_str[6..];
                if let Ok(decoded) = base64::engine::general_purpose::STANDARD.decode(encoded) {
                    if let Ok(creds) = String::from_utf8(decoded) {
                        return creds == format!("{}:{}", docs_user, docs_pass);
                    }
                }
            }
        }
    }
    false // Auth failed
}

/// Return 401 Unauthorized for docs
fn docs_unauthorized() -> web::HttpResponse {
    web::HttpResponse::Unauthorized()
        .set_header("WWW-Authenticate", "Basic realm=\"Kyx API Docs\"")
        .body("Unauthorized")
}

#[ntex::main]
async fn main() -> io::Result<()> {
    // 1. Initialize Kernel (Env, Infrastructure, Services, Modules)
    let kernel = Kernel::init().await?;

    println!(
        "🚀 Kyx Kernel v{} ({}) starting on port {}...",
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

        // Signal integration
        let signal_jwt = kernel.jwt_service.clone();
        let signal_audit = kernel.audit_service.clone();
        let signal_redis = Some(kernel.redis.clone());

        web::App::new()
            .state(redis)
            .state(database)
            .wrap(web::middleware::Logger::default())
            .wrap(
                crate::core::infrastructure::cors_middleware::DynamicCors::new(
                    cors_manager.clone(),
                ),
            )
            .wrap(
                crate::core::infrastructure::rate_limit::DynamicRateLimit::new(
                    config_service.clone(),
                ),
            )
            .service(web::resource("/api-doc/openapi.json").to(|| async {
                web::HttpResponse::Ok()
                    .content_type("application/json")
                    .json(&interface::http::openapi::ApiDoc::openapi())
            }))
            .service(
                web::resource("/api/v1/docs").to(|req: web::HttpRequest| async move {
                    if check_docs_auth(&req) {
                        serve_swagger_ui()
                    } else {
                        docs_unauthorized()
                    }
                }),
            )
            .service(
                web::resource("/api/v1/redoc").to(|req: web::HttpRequest| async move {
                    if check_docs_auth(&req) {
                        serve_redoc_ui()
                    } else {
                        docs_unauthorized()
                    }
                }),
            )
            .service(
                web::scope("/api/v1")
                    .configure(move |cfg| {
                        let _ = auth_m.try_configure(cfg);
                    })
                    .configure(move |cfg| {
                        let _ = system_m.try_configure(cfg);
                    })
                    .configure(move |cfg| {
                        let _ = media_m.try_configure(cfg);
                    })
                    // Signal integration for kyx-signal token exchange
                    .configure(move |cfg| {
                        modules::signal::interface::http::routers::signal_routes(
                            cfg,
                            signal_jwt,
                            signal_audit,
                            signal_redis,
                        );
                    }),
            )
            .service(
                web::scope("/internal")
                    .wrap(
                        crate::core::infrastructure::handshake_middleware::EngineHandshake::new(
                            engine_secret,
                        ),
                    )
                    .service(web::resource("/status").to(|| async {
                        web::HttpResponse::Ok().json(&serde_json::json!({"status": "ready"}))
                    })),
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
                    .use_last_modified(true),
            )
            .service(
                web::resource("/favicon.ico")
                    .to(|| async { web::HttpResponse::NoContent().finish() }),
            )
    })
    .bind(("0.0.0.0", kernel.port))?
    .run()
    .await
}

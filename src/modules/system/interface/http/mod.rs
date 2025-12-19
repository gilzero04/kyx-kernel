use ntex::web;
use crate::core::infrastructure::config_service::ConfigService;
use crate::core::infrastructure::audit::AuditService;
use crate::modules::system::application::api_key_service::ApiKeyService;
use crate::modules::system::application::cors_service::CORSService;
use serde::Deserialize;
use serde_json::Value;
use std::sync::Arc;

// No global routes function: Handlers are registered directly in system/mod.rs

#[web::get("/info")]
async fn get_system_info() -> impl web::Responder {
    web::HttpResponse::Ok().json(&serde_json::json!({
        "module": "system",
        "description": "Kernel Plugin Orchestrator"
    }))
}

#[web::get("/status")]
pub async fn get_system_status(
    db: web::types::State<Arc<crate::core::infrastructure::database::Database>>,
    config: web::types::State<Arc<ConfigService>>,
) -> Result<web::HttpResponse, web::Error> {
    // 1. Check if installed (has at least one user)
    let count: i64 = match sqlx::query_scalar("SELECT COUNT(*) FROM auth_users")
        .fetch_one(&db.pool)
        .await 
    {
        Ok(c) => c,
        Err(e) => {
            return Ok(web::HttpResponse::InternalServerError().json(&serde_json::json!({
                "error": format!("Database error: {}", e)
            })));
        }
    };
    
    let is_installed = count > 0;

    // 2. Fetch branding metadata
    let app_name = config.get_string("app_name", "Kyx Platform").await;
    let splash_text = config.get_string("splash_text", "Kyx Platform").await;
    let theme_id = config.get_string("theme_id", "light").await;

    Ok(web::HttpResponse::Ok().json(&serde_json::json!({
        "installed": is_installed,
        "version": "1.0.0",
        "branding": {
            "app_name": app_name,
            "splash": {
                "text": splash_text,
                "subtext": "Secure Environment"
            },
            "theme_id": theme_id
        }
    })))
}

#[web::get("/admin/test")]
async fn admin_test() -> impl web::Responder {
    web::HttpResponse::Ok().json(&serde_json::json!({
        "status": "success",
        "message": "Welcome, Admin! This is a protected route."
    }))
}

#[derive(Deserialize)]
struct ConfigUpdate {
    key: String,
    value: Value,
}

#[web::get("/config")]
async fn get_config(config: web::types::State<Arc<ConfigService>>) -> impl web::Responder {
    let access_token_expire = config.get_int("access_token_expire_minutes", 30).await;
    let refresh_token_expire = config.get_int("refresh_token_expire_minutes", 1440).await;
    
    web::HttpResponse::Ok().json(&serde_json::json!({
        "access_token_expire_minutes": access_token_expire,
        "refresh_token_expire_minutes": refresh_token_expire,
    }))
}

#[web::patch("/config")]
async fn update_config(
    body: web::types::Json<ConfigUpdate>,
    config: web::types::State<Arc<ConfigService>>,
    audit: web::types::State<Arc<AuditService>>,
) -> impl web::Responder {
    let key = &body.key;
    let value = body.value.clone();
    
    if key == "refresh_token_expire_minutes" {
        if let Some(i) = value.as_i64() {
            if i > 43200 { 
                return web::HttpResponse::BadRequest().json(&serde_json::json!({
                    "error": "Refresh token expiry cannot exceed 30 days"
                }));
            }
        }
    }

    if let Err(e) = config.set(key, value.clone()).await {
        return web::HttpResponse::InternalServerError().json(&serde_json::json!({ "error": e.message }));
    }
    
    let _ = audit.log("SuperAdmin", "CONFIG_UPDATE", Some(key), "SUCCESS", Some(value)).await;
    web::HttpResponse::Ok().json(&serde_json::json!({ "status": "success" }))
}

// === API Key Management ===

#[derive(Deserialize)]
struct CreateKeyRequest {
    tenant_id: String,
    name: Option<String>,
    key_type: String, // server | client
}

#[web::post("/api-keys")]
async fn create_api_key(
    body: web::types::Json<CreateKeyRequest>,
    service: web::types::State<Arc<ApiKeyService>>,
    audit: web::types::State<Arc<AuditService>>,
) -> impl web::Responder {
    match service.create_key(&body.tenant_id, body.name.clone(), &body.key_type, None).await {
        Ok((key, plain)) => {
            let _ = audit.log("SuperAdmin", "API_KEY_CREATED", Some(&key.id.to_string()), "SUCCESS", None).await;
            web::HttpResponse::Created().json(&serde_json::json!({
                "id": key.id,
                "prefix": key.prefix,
                "plain_key": plain, // Only shown once
                "note": "Save this key, it won't be shown again!"
            }))
        },
        Err(e) => web::HttpResponse::InternalServerError().json(&serde_json::json!({ "error": e.message }))
    }
}

// === CORS Management ===

#[derive(Deserialize)]
struct AddCorsRequest {
    origin: String,
    description: Option<String>,
}

#[web::get("/cors")]
async fn list_cors_origins(service: web::types::State<Arc<CORSService>>) -> impl web::Responder {
    match service.list_origins().await {
        Ok(origins) => web::HttpResponse::Ok().json(&origins),
        Err(e) => web::HttpResponse::InternalServerError().json(&serde_json::json!({ "error": e.message }))
    }
}

#[web::post("/cors")]
async fn add_cors_origin(
    body: web::types::Json<AddCorsRequest>,
    service: web::types::State<Arc<CORSService>>,
    audit: web::types::State<Arc<AuditService>>,
) -> impl web::Responder {
    match service.add_origin(&body.origin, body.description.clone()).await {
        Ok(origin) => {
            let _ = audit.log("SuperAdmin", "CORS_ORIGIN_ADDED", Some(&origin.origin), "SUCCESS", None).await;
            web::HttpResponse::Created().json(&origin)
        },
        Err(e) => web::HttpResponse::BadRequest().json(&serde_json::json!({ "error": e.message }))
    }
}

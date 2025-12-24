use ntex::web;
use std::sync::Arc;
use crate::core::infrastructure::config_service::ConfigService;
use crate::core::infrastructure::audit::AuditService;
use crate::modules::system::interface::http::dto::config::ConfigUpdate;

/// Get system configuration
#[utoipa::path(
    get,
    path = "/api/v1/admin/config",
    responses(
        (status = 200, description = "System configuration fetched successfully")
    ),
    tag = "settings",
    security(
        ("bearer_auth" = [])
    )
)]
pub async fn get_config(config: web::types::State<Arc<ConfigService>>) -> impl web::Responder {
    let access_token_expire = config.get_int("access_token_expire_minutes", 30).await;
    let refresh_token_expire = config.get_int("refresh_token_expire_minutes", 1440).await;
    
    web::HttpResponse::Ok().json(&serde_json::json!({
        "access_token_expire_minutes": access_token_expire,
        "refresh_token_expire_minutes": refresh_token_expire,
    }))
}

/// Update system configuration
#[utoipa::path(
    patch,
    path = "/api/v1/admin/config",
    request_body = ConfigUpdate,
    responses(
        (status = 200, description = "System configuration updated successfully"),
        (status = 400, description = "Invalid configuration value"),
        (status = 500, description = "Internal server error")
    ),
    tag = "settings",
    security(
        ("bearer_auth" = [])
    )
)]
pub async fn update_config(
    body: web::types::Json<ConfigUpdate>,
    config: web::types::State<Arc<ConfigService>>,
    audit: web::types::State<Arc<AuditService>>,
) -> impl web::Responder {
    log::info!("PATCH /config hit: key={}", body.key);
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

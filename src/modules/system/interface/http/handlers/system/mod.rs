use ntex::web;
use std::sync::Arc;
use crate::core::infrastructure::config_service::ConfigService;
use crate::core::infrastructure::database::Database;
use sqlx::Row;

/// Get basic system information
#[utoipa::path(
    get,
    path = "/api/v1/public/system/info",
    responses(
        (status = 200, description = "System information fetched successfully")
    ),
    tag = "status"
)]
pub async fn get_system_info() -> impl web::Responder {
    web::HttpResponse::Ok().json(&serde_json::json!({
        "module": "system",
        "description": "Kernel Plugin Orchestrator"
    }))
}

pub async fn admin_test() -> impl web::Responder {
    web::HttpResponse::Ok().json(&serde_json::json!({
        "status": "success",
        "message": "Welcome, Admin! This is a protected route."
    }))
}

/// Get system settings (Admin)
#[utoipa::path(
    get,
    path = "/api/v1/admin/settings",
    responses(
        (status = 200, description = "System settings fetched successfully")
    ),
    tag = "settings",
    security(
        ("bearer_auth" = [])
    )
)]
pub async fn get_system_settings(
    config: web::types::State<Arc<ConfigService>>,
) -> impl web::Responder {
    // Security settings
    let rate_limit_max = config.get_int("rate_limit_max_requests", 100).await;
    let rate_limit_window = config.get_int("rate_limit_window_secs", 60).await;
    // Use same keys as AuthModule
    let token_access_expiry = config.get_int("access_token_expire_minutes", 30).await;
    let token_refresh_expiry = config.get_int("refresh_token_expire_minutes", 1440).await; // 1440 min = 24 hours

    // Frontend settings
    let toast_position = config.get_string("toast_position", "bottom-right").await;

    // AI Translation Settings
    let ai_enabled_str = config.get_string("ai_enabled", "false").await;
    let ai_enabled = ai_enabled_str == "true";
    let ai_provider = config.get_string("ai_provider", "openai").await;
    let ai_api_key = config.get_string("ai_api_key", "").await;
    let ai_model = config.get_string("ai_model", "gpt-4o").await;
    let ai_base_url = config.get_string("ai_base_url", "https://api.openai.com/v1").await;

    web::HttpResponse::Ok().json(&serde_json::json!({
        "security": {
            "rate_limit": {
                "max_requests": rate_limit_max,
                "window_secs": rate_limit_window
            },
            "token_expiry": {
                "access_min": token_access_expiry,
                "refresh_min": token_refresh_expiry
            }
        },
        "frontend": {
            "toast_position": toast_position
        },
        "ai": {
            "enabled": ai_enabled,
            "provider": ai_provider,
            "api_key": ai_api_key,
            "model": ai_model,
            "base_url": ai_base_url
        }
    }))
}

/// Get system status and branding
#[utoipa::path(
    get,
    path = "/api/v1/system/status",
    responses(
        (status = 200, description = "System status fetched successfully")
    ),
    tag = "status"
)]
pub async fn get_system_status(
    db: web::types::State<Arc<Database>>,
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
    let app_name = config.get_string("branding_app_name", "").await;
    let splash_text = config.get_string("branding_splash_text", "").await;
    let splash_init_text = config.get_string("branding_splash_init_text", "").await;
    let theme_light_id = config.get_string("theme_light_id", "light").await;
    let theme_dark_id = config.get_string("theme_dark_id", "dark").await;
    let logo_url = config.get_string("branding_logo_url", "").await;
    let logo_dark_url = config.get_string("branding_logo_dark_url", "").await;

    // 3. Fetch owner info from auth_tenants (owner = parent_id IS NULL)
    let owner_row = sqlx::query("SELECT id, name, slug FROM auth_tenants WHERE parent_id IS NULL AND deleted_at IS NULL LIMIT 1")
        .fetch_optional(&db.pool)
        .await
        .ok()
        .flatten();
    
    let (_owner_id, owner_name, owner_slug): (Option<String>, Option<String>, Option<String>) = match owner_row {
        Some(row) => (
            Some(row.get::<sqlx::types::Uuid, _>("id").to_string()),
            Some(row.get::<String, _>("name")),
            Some(row.get::<String, _>("slug"))
        ),
        None => (None, None, None)
    };
    
    // Additional owner info from sys_configs
    let owner_address = config.get_string("owner_address", "").await;
    let owner_tax_id = config.get_string("owner_tax_id", "").await;

    Ok(web::HttpResponse::Ok().json(&serde_json::json!({
        "installed": is_installed,
        "status": "online",
        // version and owner.id removed from public response for security
        "branding": {
            "app_name": if app_name.is_empty() { None } else { Some(app_name) },
            "splash": {
                "text": if splash_text.is_empty() { None } else { Some(splash_text) },
                "subtext": if splash_init_text.is_empty() { None } else { Some(splash_init_text) }
            },
            "theme_light_id": theme_light_id,
            "theme_dark_id": theme_dark_id,
            "logo": if logo_url.is_empty() { None } else { Some(logo_url) },
            "logo_dark": if logo_dark_url.is_empty() { None } else { Some(logo_dark_url) }
        },
        "owner": {
            // id removed from public response for security
            "name": owner_name,
            "slug": owner_slug,
            "address": if owner_address.is_empty() { None } else { Some(owner_address) },
            "tax_id": if owner_tax_id.is_empty() { None } else { Some(owner_tax_id) }
        }
    })))
}

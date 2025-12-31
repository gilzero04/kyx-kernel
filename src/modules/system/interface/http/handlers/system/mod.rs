use ntex::web;
use std::sync::Arc;
use crate::core::infrastructure::config_service::ConfigService;
use crate::core::infrastructure::database::Database;
use sqlx::Row;
use crate::modules::system::application::services::tenant::TenantService;

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
    let theme_light_id = config.get_string("theme_light_id", "").await;
    let theme_dark_id = config.get_string("theme_dark_id", "").await;
    let logo_url = config.get_string("branding_logo_url", "").await;
    let logo_dark_url = config.get_string("branding_logo_dark_url", "").await;
    let favicon_url = config.get_string("branding_favicon_console", "").await;
    let icon_app_url = config.get_string("branding_icon_app", "").await;

    // 3. Fetch owner info from auth_tenants (owner = parent_id = id)
    let owner_row = sqlx::query("SELECT id, name, slug, logo_url, logo_dark_url, favicon_url, icon_app_url, primary_color, secondary_color, accent_color, custom_domain, allow_child_subdomains, domain_verified_at, verification_token FROM auth_tenants WHERE parent_id = id AND deleted_at IS NULL LIMIT 1")
        .fetch_optional(&db.pool)
        .await
        .ok()
        .flatten();
    
    let (owner_id, owner_name, owner_slug, o_logo, o_logo_dark, o_favicon, o_icon_app, p_color, s_color, a_color, c_domain, a_children, d_verified, v_token): (Option<String>, Option<String>, Option<String>, Option<String>, Option<String>, Option<String>, Option<String>, Option<String>, Option<String>, Option<String>, Option<String>, Option<bool>, Option<chrono::DateTime<chrono::Utc>>, Option<String>) = match owner_row {
        Some(row) => (
            Some(row.get::<sqlx::types::Uuid, _>("id").to_string()),
            Some(row.get::<String, _>("name")),
            Some(row.get::<String, _>("slug")),
            row.get::<Option<String>, _>("logo_url"),
            row.get::<Option<String>, _>("logo_dark_url"),
            row.get::<Option<String>, _>("favicon_url"),
            row.get::<Option<String>, _>("icon_app_url"),
            row.get::<Option<String>, _>("primary_color"),
            row.get::<Option<String>, _>("secondary_color"),
            row.get::<Option<String>, _>("accent_color"),
            row.get::<Option<String>, _>("custom_domain"),
            row.get::<Option<bool>, _>("allow_child_subdomains"),
            row.get::<Option<chrono::DateTime<chrono::Utc>>, _>("domain_verified_at"),
            row.get::<Option<String>, _>("verification_token")
        ),
        None => (None, None, None, None, None, None, None, None, None, None, None, None, None, None)
    };
    
    // Additional owner info from sys_configs
    let owner_address = config.get_string("owner_address", "").await;
    let owner_tax_id = config.get_string("owner_tax_id", "").await;

    Ok(web::HttpResponse::Ok().json(&serde_json::json!({
        "installed": is_installed,
        "status": "online",
        "branding": {
            "app_name": if app_name.is_empty() { None } else { Some(app_name) },
            "splash": {
                "text": if splash_text.is_empty() { None } else { Some(splash_text) },
                "subtext": if splash_init_text.is_empty() { None } else { Some(splash_init_text) }
            },
            "theme_light_id": theme_light_id,
            "theme_dark_id": theme_dark_id,
            "logo": if logo_url.is_empty() { o_logo } else { Some(logo_url) },
            "logo_dark": if logo_dark_url.is_empty() { o_logo_dark } else { Some(logo_dark_url) },
            "favicon_console": if favicon_url.is_empty() { o_favicon } else { Some(favicon_url) },
            "icon_app": if icon_app_url.is_empty() { o_icon_app } else { Some(icon_app_url) },
            "primary_color": p_color,
            "secondary_color": s_color,
            "accent_color": a_color
        },
        "owner": {
            "id": owner_id, // Required for CMS public page routing
            "name": owner_name,
            "slug": owner_slug,
            "address": if owner_address.is_empty() { None } else { Some(owner_address) },
            "tax_id": if owner_tax_id.is_empty() { None } else { Some(owner_tax_id) },
            "custom_domain": c_domain,
            "allow_child_subdomains": a_children,
            "domain_verified_at": d_verified,
            "verification_token": v_token
        }
    })))
}

/// Get authenticated system context
#[utoipa::path(
    get,
    path = "/api/v1/system/context",
    responses(
        (status = 200, description = "System context fetched successfully")
    ),
    tag = "status",
    security(
        ("bearer_auth" = [])
    )
)]
pub async fn get_system_context(
    tenant_service: web::types::State<Arc<TenantService>>,
) -> impl web::Responder {
    match tenant_service.get_owner_id().await {
        Ok(owner_id) => web::HttpResponse::Ok().json(&serde_json::json!({
            "owner_id": owner_id,
        })),
        Err(e) => web::HttpResponse::InternalServerError().json(&serde_json::json!({
            "error": e.message
        }))
    }
}

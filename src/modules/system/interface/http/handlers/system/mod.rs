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
    path = "/api/v1/public/system/status",
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

    // If not installed, return minimal response (Option A)
    if !is_installed {
        return Ok(web::HttpResponse::Ok().json(&serde_json::json!({
            "installed": false,
            "status": "online"
        })));
    }

    // 2. Fetch owner tenant with branding from sys_brandings (single source of truth)
    // JOIN with sys_themes to get theme codes (portable identifiers) for each context
    let owner_row = sqlx::query(r#"
        SELECT 
            t.id,
            t.name,
            t.slug,
            t.custom_domain,
            t.allow_child_subdomains,
            t.domain_verified_at,
            t.verification_token,
            b.app_name,
            b.logo_light_url,
            b.logo_dark_url,
            b.favicon_url,
            b.icon_app_url,
            b.primary_color,
            b.secondary_color,
            b.accent_color,
            b.splash_text,
            b.splash_subtext,
            -- Theme slugs: theme_light_id/theme_dark_id are console defaults
            tl.slug AS theme_light_slug,
            td.slug AS theme_dark_slug,
            -- Per-context theme overrides (fallback to console default)
            COALESCE(twl.slug, tl.slug) AS theme_workspace_light_slug,
            COALESCE(twd.slug, td.slug) AS theme_workspace_dark_slug,
            COALESCE(tal.slug, twl.slug, tl.slug) AS theme_app_light_slug,
            COALESCE(tad.slug, twd.slug, td.slug) AS theme_app_dark_slug
        FROM auth_tenants t
        LEFT JOIN sys_brandings b ON t.branding_id = b.id
        -- Theme joins (console uses theme_light_id/theme_dark_id directly)
        LEFT JOIN sys_themes tl ON b.theme_light_id = tl.id
        LEFT JOIN sys_themes td ON b.theme_dark_id = td.id
        LEFT JOIN sys_themes twl ON b.theme_workspace_light_id = twl.id
        LEFT JOIN sys_themes twd ON b.theme_workspace_dark_id = twd.id
        LEFT JOIN sys_themes tal ON b.theme_app_light_id = tal.id
        LEFT JOIN sys_themes tad ON b.theme_app_dark_id = tad.id
        WHERE t.parent_id = t.id AND t.deleted_at IS NULL
        LIMIT 1
    "#)
        .fetch_optional(&db.pool)
        .await
        .ok()
        .flatten();
    
    // Extract theme slugs (console uses theme_light_slug/theme_dark_slug)
    let (
        owner_id, owner_name, owner_slug, 
        c_domain, a_children, d_verified, v_token,
        app_name, logo, logo_dark, favicon, icon_app,
        p_color, s_color, a_color,
        splash_text, splash_subtext,
        theme_light_slug, theme_dark_slug,
        theme_workspace_light, theme_workspace_dark,
        theme_app_light, theme_app_dark
    ): (
        Option<String>, Option<String>, Option<String>,
        Option<String>, Option<bool>, Option<chrono::DateTime<chrono::Utc>>, Option<String>,
        Option<String>, Option<String>, Option<String>, Option<String>, Option<String>,
        Option<String>, Option<String>, Option<String>,
        Option<String>, Option<String>,
        Option<String>, Option<String>,
        Option<String>, Option<String>,
        Option<String>, Option<String>
    ) = match owner_row {
        Some(row) => (
            Some(row.get::<sqlx::types::Uuid, _>("id").to_string()),
            Some(row.get::<String, _>("name")),
            Some(row.get::<String, _>("slug")),
            row.get::<Option<String>, _>("custom_domain"),
            row.get::<Option<bool>, _>("allow_child_subdomains"),
            row.get::<Option<chrono::DateTime<chrono::Utc>>, _>("domain_verified_at"),
            row.get::<Option<String>, _>("verification_token"),
            row.get::<Option<String>, _>("app_name"),
            row.get::<Option<String>, _>("logo_light_url"),
            row.get::<Option<String>, _>("logo_dark_url"),
            row.get::<Option<String>, _>("favicon_url"),
            row.get::<Option<String>, _>("icon_app_url"),
            row.get::<Option<String>, _>("primary_color"),
            row.get::<Option<String>, _>("secondary_color"),
            row.get::<Option<String>, _>("accent_color"),
            row.get::<Option<String>, _>("splash_text"),
            row.get::<Option<String>, _>("splash_subtext"),
            row.get::<Option<String>, _>("theme_light_slug"),
            row.get::<Option<String>, _>("theme_dark_slug"),
            row.get::<Option<String>, _>("theme_workspace_light_slug"),
            row.get::<Option<String>, _>("theme_workspace_dark_slug"),
            row.get::<Option<String>, _>("theme_app_light_slug"),
            row.get::<Option<String>, _>("theme_app_dark_slug"),
        ),
        None => (None, None, None, None, None, None, None, None, None, None, None, None, None, None, None, None, None, None, None, None, None, None, None)
    };
    
    // Additional owner info from sys_configs (non-branding)
    let owner_address = config.get_string("owner_address", "").await;
    let owner_tax_id = config.get_string("owner_tax_id", "").await;

    Ok(web::HttpResponse::Ok().json(&serde_json::json!({
        "installed": true,
        "status": "online",
        "branding": {
            "app_name": app_name,
            "splash": {
                "text": splash_text,
                "subtext": splash_subtext
            },
            // Legacy fields (backward compatibility) - also used as console theme
            "theme_light_id": theme_light_slug.clone().unwrap_or_default(),
            "theme_dark_id": theme_dark_slug.clone().unwrap_or_default(),
            // Per-context themes (console uses theme_light_slug directly)
            "themes": {
                "console": {
                    "light": theme_light_slug.clone().unwrap_or_default(),
                    "dark": theme_dark_slug.clone().unwrap_or_default()
                },
                "workspace": {
                    "light": theme_workspace_light.or(theme_light_slug.clone()).unwrap_or_default(),
                    "dark": theme_workspace_dark.or(theme_dark_slug.clone()).unwrap_or_default()
                },
                "app": {
                    "light": theme_app_light.or(theme_light_slug).unwrap_or_default(),
                    "dark": theme_app_dark.or(theme_dark_slug).unwrap_or_default()
                }
            },
            "logo": logo,
            "logo_dark": logo_dark,
            "favicon_console": favicon,
            "icon_app": icon_app,
            "primary_color": p_color,
            "secondary_color": s_color,
            "accent_color": a_color
        },
        "owner": {
            "id": owner_id,
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
    path = "/api/v1/admin/context",
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

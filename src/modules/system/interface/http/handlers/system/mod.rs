use crate::core::infrastructure::config_service::ConfigService;
use crate::core::infrastructure::database::Database;
use crate::core::utils::response::ApiResponse;
use crate::modules::system::application::services::tenant::TenantService;
use ntex::web;
use sqlx::Row;
use std::sync::Arc;

/// Get basic system information including version metadata
#[utoipa::path(
    get,
    path = "/api/v1/public/system/info",
    responses(
        (status = 200, description = "System information fetched successfully")
    ),
    tag = "status"
)]
pub async fn get_system_info() -> impl web::Responder {
    let version = std::env::var("APP_VERSION").unwrap_or_else(|_| "0.1.0".to_string());
    let build_date = std::env::var("BUILD_DATE").unwrap_or_else(|_| "unknown".to_string());
    let git_commit = std::env::var("GIT_COMMIT").unwrap_or_else(|_| "unknown".to_string());
    let environment = std::env::var("ENVIRONMENT").unwrap_or_else(|_| "local".to_string());

    let data = serde_json::json!({
        "name": "Kyx Kernel",
        "version": version,
        "environment": environment,
        "build": {
            "date": build_date,
            "commit": git_commit
        },
        "developer": {
            "name": "Kyx Team",
            "website": "https://kyx.tech"
        }
    });

    let response = ApiResponse::ok(data, "System information");
    web::HttpResponse::Ok().json(&response)
}

/// Check for version updates against registry
#[utoipa::path(
    get,
    path = "/api/v1/public/system/version-check",
    responses(
        (status = 200, description = "Version check completed")
    ),
    tag = "status"
)]
pub async fn version_check() -> impl web::Responder {
    let current_version = std::env::var("APP_VERSION").unwrap_or_else(|_| "0.1.0".to_string());
    let registry_url = std::env::var("VERSION_REGISTRY_URL")
        .unwrap_or_else(|_| "file://./assets/versions.json".to_string());

    // Try to read from local file or fetch from URL
    let registry_result = if registry_url.starts_with("file://") {
        let path = registry_url.trim_start_matches("file://");
        std::fs::read_to_string(path).ok()
    } else {
        // For remote URLs, would need HTTP client - for now return None
        None
    };

    let (latest_version, changelog, update_available) = match registry_result {
        Some(content) => {
            if let Ok(registry) = serde_json::from_str::<serde_json::Value>(&content) {
                let latest = registry["latest"]
                    .as_str()
                    .unwrap_or(&current_version)
                    .to_string();
                let changelog = registry["releases"]
                    .as_array()
                    .and_then(|r| r.first())
                    .and_then(|v| v["changelog"].as_str())
                    .unwrap_or("")
                    .to_string();
                let update = latest != current_version;
                (latest, changelog, update)
            } else {
                (current_version.clone(), String::new(), false)
            }
        }
        None => (current_version.clone(), String::new(), false),
    };

    web::HttpResponse::Ok().json(&serde_json::json!({
        "current_version": current_version,
        "latest_version": latest_version,
        "update_available": update_available,
        "changelog": changelog
    }))
}

pub async fn admin_test() -> impl web::Responder {
    web::HttpResponse::Ok().json(&serde_json::json!({
        "status": "success",
        "message": "Welcome, Admin! This is a protected route."
    }))
}

use crate::core::utils::jwt::Claims;

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
    req: web::HttpRequest,
) -> Result<impl web::Responder, web::Error> {
    let claims = req
        .extensions()
        .get::<Claims>()
        .cloned()
        .ok_or_else(|| web::error::ErrorUnauthorized("Missing authentication claims"))?;

    let tenant_id = claims.tenant_id;

    // Security settings
    let rate_limit_max = config
        .get_tenant_int(Some(tenant_id), "rate_limit_max_requests", 100)
        .await;
    let rate_limit_window = config
        .get_tenant_int(Some(tenant_id), "rate_limit_window_secs", 60)
        .await;
    // Use same keys as AuthModule
    let token_access_expiry = config
        .get_tenant_int(Some(tenant_id), "access_token_expire_minutes", 30)
        .await;
    let token_refresh_expiry = config
        .get_tenant_int(Some(tenant_id), "refresh_token_expire_minutes", 1440)
        .await; // 1440 min = 24 hours

    // Frontend settings (String)
    let toast_position = config
        .get_tenant_string(Some(tenant_id), "toast_position", "bottom-right")
        .await;

    // AI Translation Settings (String/Bool handled by get_tenant_string logic now)
    let ai_enabled_str = config
        .get_tenant_string(Some(tenant_id), "ai_enabled", "false")
        .await;
    let ai_enabled = ai_enabled_str == "true";
    let ai_provider = config
        .get_tenant_string(Some(tenant_id), "ai_provider", "openai")
        .await;
    let ai_api_key = config
        .get_tenant_string(Some(tenant_id), "ai_api_key", "")
        .await;
    let ai_model = config
        .get_tenant_string(Some(tenant_id), "ai_model", "gpt-4o")
        .await;
    let ai_base_url = config
        .get_tenant_string(Some(tenant_id), "ai_base_url", "https://api.openai.com/v1")
        .await;

    Ok(web::HttpResponse::Ok().json(&serde_json::json!({
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
    })))
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
            return Ok(
                web::HttpResponse::InternalServerError().json(&serde_json::json!({
                    "error": format!("Database error: {}", e)
                })),
            );
        }
    };

    let is_installed = count > 0;

    // If not installed, return minimal response (Option A)
    if !is_installed {
        let data = serde_json::json!({
            "installed": false,
            "status": "online"
        });
        let response = ApiResponse::ok(data, "System not installed");
        return Ok(web::HttpResponse::Ok().json(&response));
    }

    // 2. Fetch owner tenant with branding from sys_brandings (single source of truth)
    // JOIN with sys_themes to get theme codes (portable identifiers) for each context
    let owner_row = sqlx::query(
        r#"
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
    "#,
    )
    .fetch_optional(&db.pool)
    .await
    .ok()
    .flatten();

    // Extract theme slugs (console uses theme_light_slug/theme_dark_slug)
    let (
        _owner_id,
        _owner_name,
        _owner_slug,
        _c_domain,
        _a_children,
        _d_verified,
        _v_token,
        _app_name,
        _logo,
        _logo_dark,
        _favicon,
        _icon_app,
        _splash_text,
        _splash_subtext,
        _theme_light_slug,
        _theme_dark_slug,
        _theme_workspace_light,
        _theme_workspace_dark,
        _theme_app_light,
        _theme_app_dark,
    ): (
        Option<String>,
        Option<String>,
        Option<String>,
        Option<String>,
        Option<bool>,
        Option<chrono::DateTime<chrono::Utc>>,
        Option<String>,
        Option<String>,
        Option<String>,
        Option<String>,
        Option<String>,
        Option<String>,
        Option<String>,
        Option<String>,
        Option<String>,
        Option<String>,
        Option<String>,
        Option<String>,
        Option<String>,
        Option<String>,
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
            row.get::<Option<String>, _>("splash_text"),
            row.get::<Option<String>, _>("splash_subtext"),
            row.get::<Option<String>, _>("theme_light_slug"),
            row.get::<Option<String>, _>("theme_dark_slug"),
            row.get::<Option<String>, _>("theme_workspace_light_slug"),
            row.get::<Option<String>, _>("theme_workspace_dark_slug"),
            row.get::<Option<String>, _>("theme_app_light_slug"),
            row.get::<Option<String>, _>("theme_app_dark_slug"),
        ),
        None => (
            None, None, None, None, None, None, None, None, None, None, None, None, None, None,
            None, None, None, None, None, None,
        ),
    };

    // Additional owner info from sys_configs (non-branding)
    let _owner_address = config.get_string("owner_address", "").await;
    let _owner_tax_id = config.get_string("owner_tax_id", "").await;

    let data = serde_json::json!({
        "installed": true,
        "status": "online"
    });
    let response = ApiResponse::ok(data, "System status");
    Ok(web::HttpResponse::Ok().json(&response))
}

/// Get detailed system branding and owner information
#[utoipa::path(
    get,
    path = "/api/v1/public/system/branding",
    responses(
        (status = 200, description = "System branding fetched successfully")
    ),
    tag = "status"
)]
pub async fn get_system_branding(
    db: web::types::State<Arc<Database>>,
    config: web::types::State<Arc<ConfigService>>,
) -> Result<web::HttpResponse, web::Error> {
    // 1. Fetch owner tenant with branding from sys_brandings
    let owner_row = sqlx::query(
        r#"
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
        -- Theme joins
        LEFT JOIN sys_themes tl ON b.theme_light_id = tl.id
        LEFT JOIN sys_themes td ON b.theme_dark_id = td.id
        LEFT JOIN sys_themes twl ON b.theme_workspace_light_id = twl.id
        LEFT JOIN sys_themes twd ON b.theme_workspace_dark_id = twd.id
        LEFT JOIN sys_themes tal ON b.theme_app_light_id = tal.id
        LEFT JOIN sys_themes tad ON b.theme_app_dark_id = tad.id
        WHERE t.parent_id = t.id AND t.deleted_at IS NULL
        LIMIT 1
    "#,
    )
    .fetch_optional(&db.pool)
    .await
    .ok()
    .flatten();

    // Extract data
    let (
        owner_id,
        owner_name,
        owner_slug,
        c_domain,
        a_children,
        d_verified,
        v_token,
        app_name,
        logo,
        logo_dark,
        favicon,
        icon_app,
        splash_text,
        splash_subtext,
        theme_light_slug,
        theme_dark_slug,
        theme_workspace_light,
        theme_workspace_dark,
        theme_app_light,
        theme_app_dark,
    ): (
        Option<String>,
        Option<String>,
        Option<String>,
        Option<String>,
        Option<bool>,
        Option<chrono::DateTime<chrono::Utc>>,
        Option<String>,
        Option<String>,
        Option<String>,
        Option<String>,
        Option<String>,
        Option<String>,
        Option<String>,
        Option<String>,
        Option<String>,
        Option<String>,
        Option<String>,
        Option<String>,
        Option<String>,
        Option<String>,
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
            row.get::<Option<String>, _>("splash_text"),
            row.get::<Option<String>, _>("splash_subtext"),
            row.get::<Option<String>, _>("theme_light_slug"),
            row.get::<Option<String>, _>("theme_dark_slug"),
            row.get::<Option<String>, _>("theme_workspace_light_slug"),
            row.get::<Option<String>, _>("theme_workspace_dark_slug"),
            row.get::<Option<String>, _>("theme_app_light_slug"),
            row.get::<Option<String>, _>("theme_app_dark_slug"),
        ),
        None => (
            None, None, None, None, None, None, None, None, None, None, None, None, None, None,
            None, None, None, None, None, None,
        ),
    };

    // Additional owner info from sys_configs (non-branding)
    let owner_address = config.get_string("owner_address", "").await;
    let owner_tax_id = config.get_string("owner_tax_id", "").await;

    let branding_data = serde_json::json!({
        "branding": {
            "app_name": app_name,
            "splash": {
                "text": splash_text,
                "subtext": splash_subtext
            },
            // Backend-provided default theme codes
            "theme_light_id": theme_light_slug.clone().unwrap_or_default(),
            "theme_dark_id": theme_dark_slug.clone().unwrap_or_default(),
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
    });
    let response = ApiResponse::ok(branding_data, "System branding retrieved");
    Ok(web::HttpResponse::Ok().json(&response))
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
        Ok(owner_id) => {
            let response = ApiResponse::ok(
                serde_json::json!({ "owner_id": owner_id }),
                "System context retrieved",
            );
            web::HttpResponse::Ok().json(&response)
        }
        Err(e) => {
            let response = ApiResponse::<()>::internal_error(&e.message);
            web::HttpResponse::InternalServerError().json(&response)
        }
    }
}

use crate::core::infrastructure::config_service::ConfigService;
use crate::core::infrastructure::database::Database;
use crate::core::utils::response::ApiResponse;
use ntex::web;
use serde::Deserialize;
use sqlx::Row;
use std::sync::Arc;

/// Query parameters for unified branding endpoint
#[derive(Debug, Deserialize)]
pub struct BrandingQuery {
    /// Tenant UUID - lookup by ID
    pub tenant_id: Option<String>,
    /// Tenant slug - lookup by slug
    pub slug: Option<String>,
    /// Custom domain - lookup by domain
    pub domain: Option<String>,
    /// Context: 'console' or 'workspace' (default: console)
    pub context: Option<String>,
}

/// Unified branding endpoint - replaces system/branding and workspace/status
///
/// Query Parameters:
/// - tenant_id: UUID of tenant to get branding for
/// - slug: Tenant slug to get branding for
/// - domain: Custom domain to lookup tenant
/// - context: 'console' or 'workspace' (default: console)
///
/// If no params provided, returns Platform Owner branding.
#[utoipa::path(
    get,
    path = "/api/v1/public/branding",
    params(
        ("tenant_id" = Option<String>, Query, description = "Tenant UUID"),
        ("slug" = Option<String>, Query, description = "Tenant slug"),
        ("domain" = Option<String>, Query, description = "Custom domain"),
        ("context" = Option<String>, Query, description = "Context: 'console' or 'workspace'")
    ),
    responses(
        (status = 200, description = "Branding retrieved successfully"),
        (status = 404, description = "Tenant not found")
    ),
    tag = "branding"
)]
pub async fn get_branding(
    db: web::types::State<Arc<Database>>,
    _config: web::types::State<Arc<ConfigService>>,
    query: web::types::Query<BrandingQuery>,
) -> Result<web::HttpResponse, web::Error> {
    let context = query.context.as_deref().unwrap_or("console");

    // Build WHERE clause based on query params
    let (where_clause, bind_value): (&str, Option<String>) =
        if let Some(ref tenant_id) = query.tenant_id {
            ("t.id = $1", Some(tenant_id.clone()))
        } else if let Some(ref slug) = query.slug {
            ("t.slug = $1", Some(slug.clone()))
        } else if let Some(ref domain) = query.domain {
            ("t.custom_domain = $1", Some(domain.clone()))
        } else {
            // Default: Platform Owner (parent_id = self)
            ("t.parent_id = t.id", None)
        };

    // Build SQL query - only fetch public-safe fields
    let sql = format!(
        r#"
        SELECT 
            t.id,
            t.name,
            t.slug,
            t.is_active,
            tt.slug as tenant_type,
            b.app_name,
            b.logo_light_url,
            b.logo_dark_url,
            b.favicon_url,
            b.icon_app_url,
            b.splash_text,
            b.splash_subtext,
            -- Theme slugs
            tl.slug AS theme_light_slug,
            td.slug AS theme_dark_slug,
            COALESCE(twl.slug, tl.slug) AS theme_workspace_light_slug,
            COALESCE(twd.slug, td.slug) AS theme_workspace_dark_slug,
            COALESCE(tal.slug, twl.slug, tl.slug) AS theme_app_light_slug,
            COALESCE(tad.slug, twd.slug, td.slug) AS theme_app_dark_slug
        FROM auth_tenants t
        LEFT JOIN sys_tenant_types tt ON t.tenant_type_id = tt.id
        LEFT JOIN sys_brandings b ON t.branding_id = b.id
        LEFT JOIN sys_themes tl ON b.theme_light_id = tl.id
        LEFT JOIN sys_themes td ON b.theme_dark_id = td.id
        LEFT JOIN sys_themes twl ON b.theme_workspace_light_id = twl.id
        LEFT JOIN sys_themes twd ON b.theme_workspace_dark_id = twd.id
        LEFT JOIN sys_themes tal ON b.theme_app_light_id = tal.id
        LEFT JOIN sys_themes tad ON b.theme_app_dark_id = tad.id
        WHERE {} AND t.deleted_at IS NULL
        LIMIT 1
    "#,
        where_clause
    );

    // Execute query
    let row = if let Some(ref val) = bind_value {
        // For tenant_id, parse as UUID; for slug/domain use string directly
        if query.tenant_id.is_some() {
            let uuid = match uuid::Uuid::parse_str(val) {
                Ok(u) => u,
                Err(_) => {
                    let response = ApiResponse::<()>::bad_request("Invalid tenant_id format");
                    return Ok(web::HttpResponse::BadRequest().json(&response));
                }
            };
            sqlx::query(&sql).bind(uuid).fetch_optional(&db.pool).await
        } else {
            sqlx::query(&sql).bind(val).fetch_optional(&db.pool).await
        }
    } else {
        sqlx::query(&sql).fetch_optional(&db.pool).await
    };

    let row = match row {
        Ok(Some(r)) => r,
        Ok(None) => {
            let response = ApiResponse::<()>::not_found("Tenant not found");
            return Ok(web::HttpResponse::NotFound().json(&response));
        }
        Err(e) => {
            let response = ApiResponse::<()>::internal_error(&format!("Database error: {}", e));
            return Ok(web::HttpResponse::InternalServerError().json(&response));
        }
    };

    // Extract values - public safe fields only
    let tenant_id: sqlx::types::Uuid = row.get("id");
    let tenant_name: String = row.get("name");
    let tenant_slug: String = row.get("slug");
    let tenant_type: Option<String> = row.get("tenant_type");
    let is_active: bool = row.get("is_active");

    // Branding
    let app_name: Option<String> = row.get("app_name");
    let logo: Option<String> = row.get("logo_light_url");
    let logo_dark: Option<String> = row.get("logo_dark_url");
    let favicon: Option<String> = row.get("favicon_url");
    let icon_app: Option<String> = row.get("icon_app_url");
    let splash_text: Option<String> = row.get("splash_text");
    let splash_subtext: Option<String> = row.get("splash_subtext");

    // Theme slugs
    let theme_light_slug: Option<String> = row.get("theme_light_slug");
    let theme_dark_slug: Option<String> = row.get("theme_dark_slug");
    let theme_workspace_light: Option<String> = row.get("theme_workspace_light_slug");
    let theme_workspace_dark: Option<String> = row.get("theme_workspace_dark_slug");
    let theme_app_light: Option<String> = row.get("theme_app_light_slug");
    let theme_app_dark: Option<String> = row.get("theme_app_dark_slug");

    // Build response - public safe data only (no sensitive fields)
    let branding_data = serde_json::json!({
        "tenant": {
            "id": tenant_id.to_string(),
            "name": tenant_name,
            "slug": tenant_slug,
            "type": tenant_type,
            "is_active": is_active
        },
        "branding": {
            "app_name": app_name,
            "splash": {
                "text": splash_text,
                "subtext": splash_subtext
            },
            "logo": logo,
            "logo_dark": logo_dark,
            "favicon": favicon,
            "icon_app": icon_app,
            // Theme defaults (console)
            "theme_light_id": theme_light_slug.clone().unwrap_or_default(),
            "theme_dark_id": theme_dark_slug.clone().unwrap_or_default(),
            // Per-context themes
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
            }
        },
        "context": context
    });

    let response = ApiResponse::ok(branding_data, "Branding retrieved");
    Ok(web::HttpResponse::Ok().json(&response))
}

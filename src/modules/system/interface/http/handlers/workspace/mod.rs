use crate::core::infrastructure::database::Database;
use crate::core::utils::response::ApiResponse;
use ntex::web;
use sqlx::Row;
use std::sync::Arc;

/// Get workspace status and branding for a specific tenant
///
/// This endpoint provides tenant-specific branding for workspace context.
/// Branding is now stored in sys_brandings table (single source of truth).
#[utoipa::path(
    get,
    path = "/api/v1/public/workspace/status",
    params(
        ("tenant_id" = String, Query, description = "The tenant ID to get branding for")
    ),
    responses(
        (status = 200, description = "Workspace status fetched successfully"),
        (status = 400, description = "Missing tenant_id parameter"),
        (status = 404, description = "Tenant not found")
    ),
    tag = "workspace"
)]
pub async fn get_workspace_status(
    db: web::types::State<Arc<Database>>,
    query: web::types::Query<WorkspaceStatusQuery>,
) -> Result<web::HttpResponse, web::Error> {
    // Validate tenant_id
    let tenant_id = match &query.tenant_id {
        Some(id) => id.clone(),
        None => {
            let response = ApiResponse::<()>::bad_request("Missing tenant_id parameter");
            return Ok(web::HttpResponse::BadRequest().json(&response));
        }
    };

    // Parse UUID
    let tenant_uuid = match uuid::Uuid::parse_str(&tenant_id) {
        Ok(id) => id,
        Err(_) => {
            let response = ApiResponse::<()>::bad_request("Invalid tenant_id format");
            return Ok(web::HttpResponse::BadRequest().json(&response));
        }
    };

    // Fetch tenant with branding from sys_brandings (single source of truth)
    // JOIN with sys_themes to get theme codes (portable identifiers) for each context
    let tenant_row = sqlx::query(
        r#"
        SELECT 
            t.id,
            t.name,
            t.slug,
            t.is_active,
            tt.slug as tenant_type,
            b.name as branding_name,
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
        LEFT JOIN sys_tenant_types tt ON t.tenant_type_id = tt.id
        LEFT JOIN sys_brandings b ON t.branding_id = b.id
        -- Theme joins (console uses theme_light_id/theme_dark_id directly)
        LEFT JOIN sys_themes tl ON b.theme_light_id = tl.id
        LEFT JOIN sys_themes td ON b.theme_dark_id = td.id
        LEFT JOIN sys_themes twl ON b.theme_workspace_light_id = twl.id
        LEFT JOIN sys_themes twd ON b.theme_workspace_dark_id = twd.id
        LEFT JOIN sys_themes tal ON b.theme_app_light_id = tal.id
        LEFT JOIN sys_themes tad ON b.theme_app_dark_id = tad.id
        WHERE t.id = $1 AND t.deleted_at IS NULL
    "#,
    )
    .bind(tenant_uuid)
    .fetch_optional(&db.pool)
    .await;

    let tenant_row = match tenant_row {
        Ok(Some(row)) => row,
        Ok(None) => {
            let response = ApiResponse::<()>::not_found("Tenant not found");
            return Ok(web::HttpResponse::NotFound().json(&response));
        }
        Err(e) => {
            let response = ApiResponse::<()>::internal_error(&format!("Database error: {}", e));
            return Ok(web::HttpResponse::InternalServerError().json(&response));
        }
    };

    // Extract values
    let tenant_name: String = tenant_row.get("name");
    let tenant_slug: String = tenant_row.get("slug");
    let tenant_type: Option<String> = tenant_row.get("tenant_type");
    let is_active: bool = tenant_row.get("is_active");

    // Branding from sys_brandings
    let app_name: Option<String> = tenant_row.get("app_name");
    let logo: Option<String> = tenant_row.get("logo_light_url");
    let logo_dark: Option<String> = tenant_row.get("logo_dark_url");
    let favicon: Option<String> = tenant_row.get("favicon_url");
    let icon_app: Option<String> = tenant_row.get("icon_app_url");
    let primary_color: Option<String> = tenant_row.get("primary_color");
    let secondary_color: Option<String> = tenant_row.get("secondary_color");
    let accent_color: Option<String> = tenant_row.get("accent_color");
    let splash_text: Option<String> = tenant_row.get("splash_text");
    let splash_subtext: Option<String> = tenant_row.get("splash_subtext");

    // Theme slugs (console uses theme_light_slug/theme_dark_slug)
    let theme_light_slug: Option<String> = tenant_row.get("theme_light_slug");
    let theme_dark_slug: Option<String> = tenant_row.get("theme_dark_slug");
    let theme_workspace_light: Option<String> = tenant_row.get("theme_workspace_light_slug");
    let theme_workspace_dark: Option<String> = tenant_row.get("theme_workspace_dark_slug");
    let theme_app_light: Option<String> = tenant_row.get("theme_app_light_slug");
    let theme_app_dark: Option<String> = tenant_row.get("theme_app_dark_slug");

    Ok(web::HttpResponse::Ok().json(&serde_json::json!({
        "tenant": {
            "id": tenant_id,
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
            "primary_color": primary_color,
            "secondary_color": secondary_color,
            "accent_color": accent_color,
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
            }
        }
    })))
}

/// Query parameters for workspace status
#[derive(Debug, serde::Deserialize)]
pub struct WorkspaceStatusQuery {
    pub tenant_id: Option<String>,
}

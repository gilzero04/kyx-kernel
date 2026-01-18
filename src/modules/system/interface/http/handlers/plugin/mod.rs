// ════════════════════════════════════════════════════════════════════════════
// Plugin HTTP Handlers - REST API for Plugin Management
// ════════════════════════════════════════════════════════════════════════════

use ntex::web;
use serde::{Deserialize, Serialize};
use serde_json::json;
use std::sync::Arc;
use uuid::Uuid;

use crate::core::utils::jwt::Claims;
use crate::core::utils::response::ApiResponse;
use crate::modules::system::application::services::tenant::TenantService;
use crate::modules::system::domain::plugin::entity::MenuExtension;
use crate::modules::system::domain::plugin::registry::PluginRegistry;
use crate::modules::system::domain::plugin::{Manifest, Plugin};

// ════════════════════════════════════════════════════════════════════════════
// Request/Response DTOs
// ════════════════════════════════════════════════════════════════════════════

#[derive(Debug, Deserialize, utoipa::ToSchema)]
pub struct TenantQuery {
    pub tenant_id: Option<Uuid>,
}

#[derive(Debug, Deserialize, utoipa::ToSchema)]
pub struct InstallPluginRequest {
    pub tenant_id: Uuid,
    pub manifest: Manifest,
    pub config: Option<serde_json::Value>,
}

#[derive(Debug, Deserialize, utoipa::ToSchema)]
pub struct UpdatePluginConfigRequest {
    pub config: serde_json::Value,
}

#[derive(Debug, Serialize, utoipa::ToSchema)]
pub struct PluginResponse {
    pub id: Uuid,
    pub plugin_id: String,
    pub name: String,
    pub version: String,
    pub description: Option<String>,
    pub status: String,
    pub is_active: bool,
    pub capabilities: Vec<String>,
}

impl From<Plugin> for PluginResponse {
    fn from(p: Plugin) -> Self {
        Self {
            id: p.id,
            plugin_id: p.plugin_id,
            name: p.name,
            version: p.version,
            description: p.description,
            status: p.status,
            is_active: p.is_active,
            capabilities: p.capabilities,
        }
    }
}

#[derive(Debug, Serialize, utoipa::ToSchema)]
pub struct PluginListResponse {
    pub plugins: Vec<PluginResponse>,
    pub total: usize,
}

#[derive(Debug, Serialize, utoipa::ToSchema)]
pub struct SuccessResponse {
    pub status: String,
    pub message: String,
}

#[derive(Debug, Serialize, utoipa::ToSchema)]
pub struct ErrorResponse {
    pub error: String,
    pub message: String,
}

#[derive(Debug, Serialize, utoipa::ToSchema)]
pub struct AggregatedMenuResponse {
    pub menus: Vec<MenuExtension>,
}

// ════════════════════════════════════════════════════════════════════════════
// Handlers
// ════════════════════════════════════════════════════════════════════════════

/// List all plugins for the tenant
#[utoipa::path(
    get,
    path = "/api/v1/admin/plugins",
    params(("tenant_id" = Uuid, Query, description = "Tenant ID")),
    responses(
        (status = 200, description = "Plugins listed successfully")
    ),
    tag = "plugins",
    security(("bearer_auth" = []))
)]
pub async fn list_plugins(
    registry: web::types::State<Arc<PluginRegistry>>,
    query: web::types::Query<TenantQuery>,
    claims: Claims,
) -> web::HttpResponse {
    let tenant_id = query.tenant_id.unwrap_or(claims.tenant_id);

    match registry.list_plugins(tenant_id).await {
        Ok(plugins) => {
            let responses: Vec<PluginResponse> =
                plugins.into_iter().map(PluginResponse::from).collect();
            let total = responses.len();

            let response = ApiResponse::ok(
                json!({
                    "plugins": responses,
                    "total": total
                }),
                "Plugins listed successfully",
            );
            web::HttpResponse::Ok().json(&response)
        }
        Err(e) => {
            log::error!("Failed to list plugins: {}", e);
            let response = ApiResponse::<()>::internal_error(&e.to_string());
            web::HttpResponse::InternalServerError().json(&response)
        }
    }
}

/// Get a specific plugin by ID
#[utoipa::path(
    get,
    path = "/api/v1/admin/plugins/{id}",
    responses(
        (status = 200, description = "Plugin details returned"),
        (status = 404, description = "Plugin not found")
    ),
    tag = "plugins",
    security(("bearer_auth" = []))
)]
pub async fn get_plugin(
    registry: web::types::State<Arc<PluginRegistry>>,
    path: web::types::Path<Uuid>,
) -> web::HttpResponse {
    let plugin_id = path.into_inner();

    match registry.get_plugin(plugin_id).await {
        Ok(Some(plugin)) => {
            let response = ApiResponse::ok(
                json!({ "plugin": PluginResponse::from(plugin) }),
                "Plugin fetched successfully",
            );
            web::HttpResponse::Ok().json(&response)
        }
        Ok(None) => {
            let response = ApiResponse::<()>::not_found("Plugin not found");
            web::HttpResponse::NotFound().json(&response)
        }
        Err(e) => {
            log::error!("Failed to get plugin: {}", e);
            let response = ApiResponse::<()>::internal_error(&e.to_string());
            web::HttpResponse::InternalServerError().json(&response)
        }
    }
}

/// Install a new plugin
#[utoipa::path(
    post,
    path = "/api/v1/admin/plugins",
    request_body = InstallPluginRequest,
    responses(
        (status = 201, description = "Plugin installed successfully"),
        (status = 400, description = "Invalid plugin manifest")
    ),
    tag = "plugins",
    security(("bearer_auth" = []))
)]
pub async fn install_plugin(
    registry: web::types::State<Arc<PluginRegistry>>,
    body: web::types::Json<InstallPluginRequest>,
) -> web::HttpResponse {
    let req = body.into_inner();

    match registry
        .install(req.tenant_id, req.manifest, None, req.config)
        .await
    {
        Ok(plugin) => {
            let response = ApiResponse::created(
                json!({ "plugin": PluginResponse::from(plugin) }),
                "Plugin installed successfully",
            );
            web::HttpResponse::Created().json(&response)
        }
        Err(e) => {
            log::error!("Failed to install plugin: {}", e);
            let response = ApiResponse::<()>::bad_request(&e.to_string());
            web::HttpResponse::BadRequest().json(&response)
        }
    }
}

/// Enable a plugin
#[utoipa::path(
    post,
    path = "/api/v1/admin/plugins/{id}/enable",
    params(("tenant_id" = Uuid, Query, description = "Tenant ID")),
    responses(
        (status = 200, description = "Plugin enabled successfully"),
        (status = 404, description = "Plugin not found")
    ),
    tag = "plugins",
    security(("bearer_auth" = []))
)]
pub async fn enable_plugin(
    registry: web::types::State<Arc<PluginRegistry>>,
    path: web::types::Path<Uuid>,
    query: web::types::Query<TenantQuery>,
    claims: Claims,
) -> web::HttpResponse {
    let plugin_id = path.into_inner();
    let tenant_id = query.tenant_id.unwrap_or(claims.tenant_id);

    match registry.enable(tenant_id, plugin_id).await {
        Ok(()) => {
            let response =
                ApiResponse::ok(json!({ "enabled": true }), "Plugin enabled successfully");
            web::HttpResponse::Ok().json(&response)
        }
        Err(e) => {
            log::error!("Failed to enable plugin: {}", e);
            let response = ApiResponse::<()>::not_found(&e.to_string());
            web::HttpResponse::NotFound().json(&response)
        }
    }
}

/// Disable a plugin
#[utoipa::path(
    post,
    path = "/api/v1/admin/plugins/{id}/disable",
    params(("tenant_id" = Uuid, Query, description = "Tenant ID")),
    responses(
        (status = 200, description = "Plugin disabled successfully"),
        (status = 404, description = "Plugin not found")
    ),
    tag = "plugins",
    security(("bearer_auth" = []))
)]
pub async fn disable_plugin(
    registry: web::types::State<Arc<PluginRegistry>>,
    path: web::types::Path<Uuid>,
    query: web::types::Query<TenantQuery>,
    claims: Claims,
) -> web::HttpResponse {
    let plugin_id = path.into_inner();
    let tenant_id = query.tenant_id.unwrap_or(claims.tenant_id);

    match registry.disable(tenant_id, plugin_id).await {
        Ok(()) => {
            let response =
                ApiResponse::ok(json!({ "disabled": true }), "Plugin disabled successfully");
            web::HttpResponse::Ok().json(&response)
        }
        Err(e) => {
            log::error!("Failed to disable plugin: {}", e);
            let response = ApiResponse::<()>::not_found(&e.to_string());
            web::HttpResponse::NotFound().json(&response)
        }
    }
}

/// Uninstall a plugin
#[utoipa::path(
    delete,
    path = "/api/v1/admin/plugins/{id}",
    params(("tenant_id" = Uuid, Query, description = "Tenant ID")),
    responses(
        (status = 200, description = "Plugin uninstalled successfully"),
        (status = 404, description = "Plugin not found")
    ),
    tag = "plugins",
    security(("bearer_auth" = []))
)]
pub async fn uninstall_plugin(
    registry: web::types::State<Arc<PluginRegistry>>,
    path: web::types::Path<Uuid>,
    query: web::types::Query<TenantQuery>,
    claims: Claims,
) -> web::HttpResponse {
    let plugin_id = path.into_inner();
    let tenant_id = query.tenant_id.unwrap_or(claims.tenant_id);

    match registry.uninstall(tenant_id, plugin_id).await {
        Ok(()) => {
            let response = ApiResponse::ok(
                json!({ "uninstalled": true }),
                "Plugin uninstalled successfully",
            );
            web::HttpResponse::Ok().json(&response)
        }
        Err(e) => {
            log::error!("Failed to uninstall plugin: {}", e);
            let response = ApiResponse::<()>::bad_request(&e.to_string());
            web::HttpResponse::BadRequest().json(&response)
        }
    }
}

/// Update plugin configuration
#[utoipa::path(
    put,
    path = "/api/v1/admin/plugins/{id}/config",
    params(("tenant_id" = Uuid, Query, description = "Tenant ID")),
    request_body = UpdatePluginConfigRequest,
    responses(
        (status = 200, description = "Plugin config updated successfully"),
        (status = 404, description = "Plugin not found")
    ),
    tag = "plugins",
    security(("bearer_auth" = []))
)]
pub async fn update_plugin_config(
    registry: web::types::State<Arc<PluginRegistry>>,
    path: web::types::Path<Uuid>,
    query: web::types::Query<TenantQuery>,
    body: web::types::Json<UpdatePluginConfigRequest>,
    claims: Claims,
) -> web::HttpResponse {
    let plugin_id = path.into_inner();
    let tenant_id = query.tenant_id.unwrap_or(claims.tenant_id);
    let config = body.into_inner().config;

    match registry.update_config(tenant_id, plugin_id, config).await {
        Ok(()) => {
            let response = ApiResponse::ok(
                json!({ "updated": true }),
                "Plugin configuration updated successfully",
            );
            web::HttpResponse::Ok().json(&response)
        }
        Err(e) => {
            log::error!("Failed to update plugin config: {}", e);
            let response = ApiResponse::<()>::bad_request(&e.to_string());
            web::HttpResponse::BadRequest().json(&response)
        }
    }
}

/// Analyze plugin security before installation
#[utoipa::path(
    post,
    path = "/api/v1/admin/plugins/analyze",
    request_body = AnalyzePluginRequest,
    responses(
        (status = 200, description = "Security analysis complete")
    ),
    tag = "plugins",
    security(("bearer_auth" = []))
)]
pub async fn analyze_plugin_security(
    registry: web::types::State<Arc<PluginRegistry>>,
    body: web::types::Json<AnalyzePluginRequest>,
) -> web::HttpResponse {
    let manifest = body.into_inner().manifest;
    let summary = registry.get_security_summary(&manifest);

    let response = ApiResponse::ok(
        json!({ "security_summary": summary }),
        "Security analysis complete",
    );
    web::HttpResponse::Ok().json(&response)
}

#[derive(Debug, Deserialize, utoipa::ToSchema)]
pub struct AnalyzePluginRequest {
    pub manifest: Manifest,
}

/// Install a plugin with dangerous capabilities (requires approval)
#[utoipa::path(
    post,
    path = "/api/v1/admin/plugins/install-approved",
    request_body = InstallWithApprovalRequest,
    responses(
        (status = 201, description = "Plugin installed with approval"),
        (status = 403, description = "Forbidden - requires plugin:approve permission")
    ),
    tag = "plugins",
    security(("bearer_auth" = []))
)]
pub async fn install_plugin_with_approval(
    registry: web::types::State<Arc<PluginRegistry>>,
    body: web::types::Json<InstallWithApprovalRequest>,
) -> web::HttpResponse {
    let req = body.into_inner();

    // Permission check: Caller must have plugin:approve permission
    // NOTE: This should be enforced via RBAC middleware at the route level.
    // The approved_by UUID in the request body acts as an audit trail.
    // Route: POST /api/v1/admin/plugins/install-approved requires plugin:approve permission

    match registry
        .install_with_approval(
            req.tenant_id,
            req.manifest,
            None,
            req.config,
            req.approved_by,
            &req.approval_reason,
        )
        .await
    {
        Ok(plugin) => {
            log::info!(
                "SECURITY: Plugin {} installed with approval",
                plugin.plugin_id
            );
            web::HttpResponse::Created().json(&PluginResponse::from(plugin))
        }
        Err(e) => {
            log::error!("Failed to install plugin with approval: {}", e);
            web::HttpResponse::BadRequest().json(&ErrorResponse {
                error: "Failed to install plugin".to_string(),
                message: e.to_string(),
            })
        }
    }
}

#[derive(Debug, Deserialize, utoipa::ToSchema)]
pub struct InstallWithApprovalRequest {
    pub tenant_id: Uuid,
    pub manifest: Manifest,
    pub config: Option<serde_json::Value>,
    pub approved_by: Uuid,
    pub approval_reason: String,
}

/// Get security warnings for an installed plugin
#[utoipa::path(
    get,
    path = "/api/v1/admin/plugins/{id}/security",
    responses(
        (status = 200, description = "Security warnings returned"),
        (status = 404, description = "Plugin not found")
    ),
    tag = "plugins",
    security(("bearer_auth" = []))
)]
pub async fn get_plugin_security(
    registry: web::types::State<Arc<PluginRegistry>>,
    path: web::types::Path<Uuid>,
) -> web::HttpResponse {
    let plugin_id = path.into_inner();

    match registry.get_plugin(plugin_id).await {
        Ok(Some(plugin)) => {
            let warnings = registry.validate_plugin_security(&plugin);
            let warning_count = warnings.len();
            web::HttpResponse::Ok().json(&SecurityWarningsResponse {
                plugin_id: plugin.plugin_id,
                warnings,
                warning_count,
            })
        }
        Ok(None) => web::HttpResponse::NotFound().json(&ErrorResponse {
            error: "Not found".to_string(),
            message: "Plugin not found".to_string(),
        }),
        Err(e) => {
            log::error!("Failed to get plugin security: {}", e);
            web::HttpResponse::InternalServerError().json(&ErrorResponse {
                error: "Failed to get plugin security".to_string(),
                message: e.to_string(),
            })
        }
    }
}

#[derive(Debug, Serialize, utoipa::ToSchema)]
pub struct SecurityWarningsResponse {
    pub plugin_id: String,
    pub warnings: Vec<String>,
    pub warning_count: usize,
}

/// Get aggregated dynamic menus for current user
#[utoipa::path(
    get,
    path = "/api/v1/me/menus",
    responses(
        (status = 200, description = "Aggregated menus returned", body = AggregatedMenuResponse)
    ),
    tag = "plugins",
    security(("bearer_auth" = []))
)]
pub async fn get_menus(
    registry: web::types::State<Arc<PluginRegistry>>,
    tenant_service: web::types::State<Arc<TenantService>>,
    claims: Claims,
) -> web::HttpResponse {
    let tenant_id = claims.tenant_id;

    // 1. Get Parent Tenant ID (for inheritance)
    let parent_id = match tenant_service.get_ref().get_parent_id(tenant_id).await {
        Ok(pid) => pid,
        Err(_) => None, // If error (e.g. root tenant or db error), assume no parent
    };

    // 2. Fetch Available Plugins (Own + Global + Shared Parent)
    let plugins = match registry
        .get_ref()
        .find_available_plugins(tenant_id, parent_id)
        .await
    {
        Ok(p) => p,
        Err(e) => {
            log::error!("Failed to fetch available plugins for menus: {}", e);
            return web::HttpResponse::InternalServerError().json(&ErrorResponse {
                error: "Internal Error".to_string(),
                message: "Failed to fetch plugins".to_string(),
            });
        }
    };

    // 3. Aggregate Menus
    let mut all_menus = Vec::new();
    let user_permissions = &claims.permissions;

    for plugin in plugins {
        if let Some(ui_val) = &plugin.ui {
            if let Ok(ui) = serde_json::from_value::<
                crate::modules::system::domain::plugin::entity::UIExtensions,
            >(ui_val.clone())
            {
                for menu in ui.menus {
                    // 4. Permission Filtering
                    if menu.permissions.is_empty()
                        || menu
                            .permissions
                            .iter()
                            .any(|p| user_permissions.contains(p))
                    {
                        all_menus.push(menu);
                    }
                }
            }
        }
    }

    // 5. Sort by order
    all_menus.sort_by(|a, b| a.order.cmp(&b.order));

    web::HttpResponse::Ok().json(&AggregatedMenuResponse { menus: all_menus })
}

// ════════════════════════════════════════════════════════════════════════════
// Route Configuration
// ════════════════════════════════════════════════════════════════════════════

pub fn configure(cfg: &mut web::ServiceConfig) {
    cfg.service(
        web::scope("/plugins") // Relative path - already inside /api/v1/admin scope
            .route("", web::get().to(list_plugins))
            .route("", web::post().to(install_plugin))
            .route("/analyze", web::post().to(analyze_plugin_security))
            .route(
                "/install-approved",
                web::post().to(install_plugin_with_approval),
            )
            .route("/{id}", web::get().to(get_plugin))
            .route("/{id}", web::delete().to(uninstall_plugin))
            .route("/{id}/enable", web::post().to(enable_plugin))
            .route("/{id}/disable", web::post().to(disable_plugin))
            .route("/{id}/config", web::put().to(update_plugin_config))
            .route("/{id}/security", web::get().to(get_plugin_security)),
    );
}

pub fn configure_me(cfg: &mut web::ServiceConfig) {
    cfg.service(web::scope("/menus").route("", web::get().to(get_menus)));
}

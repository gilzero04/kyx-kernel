// ════════════════════════════════════════════════════════════════════════════
// Plugin HTTP Handlers - REST API for Plugin Management
// ════════════════════════════════════════════════════════════════════════════

use ntex::web;
use std::sync::Arc;
use uuid::Uuid;
use serde::{Deserialize, Serialize};

use crate::modules::system::domain::plugin::registry::PluginRegistry;
use crate::modules::system::domain::plugin::{Manifest, Plugin};

// ════════════════════════════════════════════════════════════════════════════
// Request/Response DTOs
// ════════════════════════════════════════════════════════════════════════════

#[derive(Debug, Deserialize)]
pub struct TenantQuery {
    pub tenant_id: Uuid,
}

#[derive(Debug, Deserialize)]
pub struct InstallPluginRequest {
    pub tenant_id: Uuid,
    pub manifest: Manifest,
    pub config: Option<serde_json::Value>,
}

#[derive(Debug, Deserialize)]
pub struct UpdatePluginConfigRequest {
    pub config: serde_json::Value,
}

#[derive(Debug, Serialize)]
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

#[derive(Debug, Serialize)]
pub struct PluginListResponse {
    pub plugins: Vec<PluginResponse>,
    pub total: usize,
}

#[derive(Debug, Serialize)]
pub struct SuccessResponse {
    pub status: String,
    pub message: String,
}

#[derive(Debug, Serialize)]
pub struct ErrorResponse {
    pub error: String,
    pub message: String,
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
) -> web::HttpResponse {
    let tenant_id = query.tenant_id;
    
    match registry.list_plugins(tenant_id).await {
        Ok(plugins) => {
            let responses: Vec<PluginResponse> = plugins.into_iter()
                .map(PluginResponse::from)
                .collect();
            let total = responses.len();
            
            web::HttpResponse::Ok().json(&PluginListResponse { 
                plugins: responses, 
                total 
            })
        }
        Err(e) => {
            log::error!("Failed to list plugins: {}", e);
            web::HttpResponse::InternalServerError().json(&ErrorResponse {
                error: "Failed to list plugins".to_string(),
                message: e.to_string()
            })
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
            web::HttpResponse::Ok().json(&PluginResponse::from(plugin))
        }
        Ok(None) => {
            web::HttpResponse::NotFound().json(&ErrorResponse {
                error: "Not found".to_string(),
                message: "Plugin not found".to_string()
            })
        }
        Err(e) => {
            log::error!("Failed to get plugin: {}", e);
            web::HttpResponse::InternalServerError().json(&ErrorResponse {
                error: "Failed to get plugin".to_string(),
                message: e.to_string()
            })
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
    
    match registry.install(req.tenant_id, req.manifest, None, req.config).await {
        Ok(plugin) => {
            web::HttpResponse::Created().json(&PluginResponse::from(plugin))
        }
        Err(e) => {
            log::error!("Failed to install plugin: {}", e);
            web::HttpResponse::BadRequest().json(&ErrorResponse {
                error: "Failed to install plugin".to_string(),
                message: e.to_string()
            })
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
) -> web::HttpResponse {
    let plugin_id = path.into_inner();
    let tenant_id = query.tenant_id;
    
    match registry.enable(tenant_id, plugin_id).await {
        Ok(()) => {
            web::HttpResponse::Ok().json(&SuccessResponse {
                status: "success".to_string(),
                message: "Plugin enabled".to_string()
            })
        }
        Err(e) => {
            log::error!("Failed to enable plugin: {}", e);
            web::HttpResponse::BadRequest().json(&ErrorResponse {
                error: "Failed to enable plugin".to_string(),
                message: e.to_string()
            })
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
) -> web::HttpResponse {
    let plugin_id = path.into_inner();
    let tenant_id = query.tenant_id;
    
    match registry.disable(tenant_id, plugin_id).await {
        Ok(()) => {
            web::HttpResponse::Ok().json(&SuccessResponse {
                status: "success".to_string(),
                message: "Plugin disabled".to_string()
            })
        }
        Err(e) => {
            log::error!("Failed to disable plugin: {}", e);
            web::HttpResponse::BadRequest().json(&ErrorResponse {
                error: "Failed to disable plugin".to_string(),
                message: e.to_string()
            })
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
) -> web::HttpResponse {
    let plugin_id = path.into_inner();
    let tenant_id = query.tenant_id;
    
    match registry.uninstall(tenant_id, plugin_id).await {
        Ok(()) => {
            web::HttpResponse::Ok().json(&SuccessResponse {
                status: "success".to_string(),
                message: "Plugin uninstalled".to_string()
            })
        }
        Err(e) => {
            log::error!("Failed to uninstall plugin: {}", e);
            web::HttpResponse::BadRequest().json(&ErrorResponse {
                error: "Failed to uninstall plugin".to_string(),
                message: e.to_string()
            })
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
) -> web::HttpResponse {
    let plugin_id = path.into_inner();
    let tenant_id = query.tenant_id;
    let config = body.into_inner().config;
    
    match registry.update_config(tenant_id, plugin_id, config).await {
        Ok(()) => {
            web::HttpResponse::Ok().json(&SuccessResponse {
                status: "success".to_string(),
                message: "Plugin configuration updated".to_string()
            })
        }
        Err(e) => {
            log::error!("Failed to update plugin config: {}", e);
            web::HttpResponse::BadRequest().json(&ErrorResponse {
                error: "Failed to update plugin configuration".to_string(),
                message: e.to_string()
            })
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
    
    web::HttpResponse::Ok().json(&summary)
}

#[derive(Debug, Deserialize)]
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
    
    // TODO: Verify caller has plugin:approve permission
    // For now, require approval_reason and approved_by in request
    
    match registry.install_with_approval(
        req.tenant_id,
        req.manifest,
        None,
        req.config,
        req.approved_by,
        &req.approval_reason,
    ).await {
        Ok(plugin) => {
            log::info!("SECURITY: Plugin {} installed with approval", plugin.plugin_id);
            web::HttpResponse::Created().json(&PluginResponse::from(plugin))
        }
        Err(e) => {
            log::error!("Failed to install plugin with approval: {}", e);
            web::HttpResponse::BadRequest().json(&ErrorResponse {
                error: "Failed to install plugin".to_string(),
                message: e.to_string()
            })
        }
    }
}

#[derive(Debug, Deserialize)]
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
        Ok(None) => {
            web::HttpResponse::NotFound().json(&ErrorResponse {
                error: "Not found".to_string(),
                message: "Plugin not found".to_string()
            })
        }
        Err(e) => {
            log::error!("Failed to get plugin security: {}", e);
            web::HttpResponse::InternalServerError().json(&ErrorResponse {
                error: "Failed to get plugin security".to_string(),
                message: e.to_string()
            })
        }
    }
}

#[derive(Debug, Serialize)]
pub struct SecurityWarningsResponse {
    pub plugin_id: String,
    pub warnings: Vec<String>,
    pub warning_count: usize,
}

// ════════════════════════════════════════════════════════════════════════════
// Route Configuration
// ════════════════════════════════════════════════════════════════════════════

pub fn configure(cfg: &mut web::ServiceConfig) {
    cfg.service(
        web::scope("/api/v1/admin/plugins")
            .route("", web::get().to(list_plugins))
            .route("", web::post().to(install_plugin))
            .route("/analyze", web::post().to(analyze_plugin_security))
            .route("/install-approved", web::post().to(install_plugin_with_approval))
            .route("/{id}", web::get().to(get_plugin))
            .route("/{id}", web::delete().to(uninstall_plugin))
            .route("/{id}/enable", web::post().to(enable_plugin))
            .route("/{id}/disable", web::post().to(disable_plugin))
            .route("/{id}/config", web::put().to(update_plugin_config))
            .route("/{id}/security", web::get().to(get_plugin_security))
    );
}

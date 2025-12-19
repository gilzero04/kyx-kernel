use ntex::web;
use crate::core::infrastructure::config_service::ConfigService;
use crate::core::infrastructure::audit::AuditService;
use crate::modules::system::application::api_key_service::ApiKeyService;
use crate::modules::system::application::cors_service::CORSService;
use serde::Deserialize;
use serde_json::Value;
use std::sync::Arc;
use sqlx::Row;

// No global routes function: Handlers are registered directly in system/mod.rs

pub mod users_handler;

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
pub async fn get_config(config: web::types::State<Arc<ConfigService>>) -> impl web::Responder {
    let access_token_expire = config.get_int("access_token_expire_minutes", 30).await;
    let refresh_token_expire = config.get_int("refresh_token_expire_minutes", 1440).await;
    
    web::HttpResponse::Ok().json(&serde_json::json!({
        "access_token_expire_minutes": access_token_expire,
        "refresh_token_expire_minutes": refresh_token_expire,
    }))
}

#[web::patch("/config")]
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

// === System Settings ===

#[web::get("/settings")]
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
        }
    }))
}

// === Owner Tenant Update ===

#[derive(Debug, Deserialize)]
pub struct UpdateOwnerRequest {
    name: Option<String>,
}

#[web::patch("/owner")]
pub async fn update_owner(
    db: web::types::State<Arc<crate::core::infrastructure::database::Database>>,
    audit: web::types::State<Arc<AuditService>>,
    body: web::types::Json<UpdateOwnerRequest>,
) -> impl web::Responder {
    // Update owner tenant (parent_id IS NULL)
    if let Some(name) = &body.name {
        let result = sqlx::query(
            "UPDATE auth_tenants SET name = $1, updated_at = NOW() WHERE parent_id IS NULL AND deleted_at IS NULL"
        )
        .bind(name)
        .execute(&db.pool)
        .await;

        match result {
            Ok(_) => {
                let _ = audit.log("SuperAdmin", "OWNER_UPDATED", Some("name"), "SUCCESS", None).await;
                web::HttpResponse::Ok().json(&serde_json::json!({ "success": true }))
            },
            Err(e) => web::HttpResponse::InternalServerError().json(&serde_json::json!({
                "error": format!("Failed to update owner: {}", e)
            }))
        }
    } else {
        web::HttpResponse::BadRequest().json(&serde_json::json!({ "error": "No fields to update" }))
    }
}

// === Audit Logs ===

use serde::Serialize;
use chrono::{DateTime, Utc};

#[derive(Debug, Serialize, sqlx::FromRow)]
pub struct AuditLogEntry {
    id: sqlx::types::Uuid,
    timestamp: DateTime<Utc>,
    actor: String,
    action: String,
    target: Option<String>,
    status: String,
    metadata: Option<Value>,
}

#[derive(Debug, Deserialize)]
pub struct LogsQuery {
    limit: Option<i64>,
    offset: Option<i64>,
    action: Option<String>,
}

#[web::get("/logs")]
pub async fn list_audit_logs(
    db: web::types::State<Arc<crate::core::infrastructure::database::Database>>,
    query: web::types::Query<LogsQuery>,
) -> impl web::Responder {
    let limit = query.limit.unwrap_or(50).min(100);
    let offset = query.offset.unwrap_or(0);

    let logs: Result<Vec<AuditLogEntry>, _> = if let Some(action_filter) = &query.action {
        sqlx::query_as::<_, AuditLogEntry>(
            "SELECT id, timestamp, actor, action, target, status, metadata 
             FROM audit_logs 
             WHERE action ILIKE $1
             ORDER BY timestamp DESC 
             LIMIT $2 OFFSET $3"
        )
        .bind(format!("%{}%", action_filter))
        .bind(limit)
        .bind(offset)
        .fetch_all(&db.pool)
        .await
    } else {
        sqlx::query_as::<_, AuditLogEntry>(
            "SELECT id, timestamp, actor, action, target, status, metadata 
             FROM audit_logs 
             ORDER BY timestamp DESC 
             LIMIT $1 OFFSET $2"
        )
        .bind(limit)
        .bind(offset)
        .fetch_all(&db.pool)
        .await
    };

    match logs {
        Ok(entries) => web::HttpResponse::Ok().json(&entries),
        Err(e) => web::HttpResponse::InternalServerError().json(&serde_json::json!({
            "error": format!("Failed to fetch logs: {}", e)
        }))
    }
}

// === Tenants Management ===

#[derive(Debug, Serialize, sqlx::FromRow)]
pub struct TenantEntry {
    id: sqlx::types::Uuid,
    name: String,
    slug: String,
    is_active: bool,
    created_at: DateTime<Utc>,
    member_count: Option<i64>,
}

#[web::get("/tenants")]
pub async fn list_tenants(
    db: web::types::State<Arc<crate::core::infrastructure::database::Database>>,
) -> impl web::Responder {
    // Filter out owner tenant (parent_id IS NULL = platform owner)
    // Only show sub-tenants (branches, vendors, etc.)
    let tenants = sqlx::query_as::<_, TenantEntry>(
        r#"
        SELECT 
            t.id, t.name, t.slug, t.is_active, t.created_at,
            (SELECT COUNT(*) FROM auth_memberships m WHERE m.tenant_id = t.id) as member_count
        FROM auth_tenants t
        WHERE t.deleted_at IS NULL 
          AND t.parent_id IS NOT NULL
        ORDER BY t.created_at DESC
        "#
    )
    .fetch_all(&db.pool)
    .await;

    match tenants {
        Ok(entries) => web::HttpResponse::Ok().json(&entries),
        Err(e) => web::HttpResponse::InternalServerError().json(&serde_json::json!({
            "error": format!("Failed to fetch tenants: {}", e)
        }))
    }
}

// === RBAC: Permissions & Roles ===

#[derive(Debug, Serialize, sqlx::FromRow)]
pub struct PermissionEntry {
    id: sqlx::types::Uuid,
    code: Option<String>,
    slug: String,
    name: String,
    description: Option<String>,
    is_active: bool,
}

#[web::get("/permissions")]
pub async fn list_permissions(
    db: web::types::State<Arc<crate::core::infrastructure::database::Database>>,
) -> impl web::Responder {
    let perms = sqlx::query_as::<_, PermissionEntry>(
        "SELECT id, code, slug, name, description, is_active 
         FROM sys_permissions WHERE deleted_at IS NULL ORDER BY sort_order, name"
    )
    .fetch_all(&db.pool)
    .await;

    match perms {
        Ok(entries) => web::HttpResponse::Ok().json(&entries),
        Err(e) => web::HttpResponse::InternalServerError().json(&serde_json::json!({
            "error": format!("Failed to fetch permissions: {}", e)
        }))
    }
}

#[derive(Debug, Serialize, sqlx::FromRow)]
pub struct RoleEntry {
    id: sqlx::types::Uuid,
    code: Option<String>,
    slug: String,
    name: String,
    is_active: bool,
    permission_count: Option<i64>,
}

#[web::get("/roles")]
pub async fn list_roles(
    db: web::types::State<Arc<crate::core::infrastructure::database::Database>>,
) -> impl web::Responder {
    let roles = sqlx::query_as::<_, RoleEntry>(
        r#"
        SELECT r.id, r.code, r.slug, r.name, r.is_active,
            (SELECT COUNT(*) FROM sys_role_permissions rp WHERE rp.role_id = r.id) as permission_count
        FROM sys_roles r
        WHERE r.deleted_at IS NULL
        ORDER BY r.sort_order DESC, r.name
        "#
    )
    .fetch_all(&db.pool)
    .await;

    match roles {
        Ok(entries) => web::HttpResponse::Ok().json(&entries),
        Err(e) => web::HttpResponse::InternalServerError().json(&serde_json::json!({
            "error": format!("Failed to fetch roles: {}", e)
        }))
    }
}

// === CRUD: Permissions ===

#[derive(Deserialize)]
pub struct CreatePermissionRequest {
    code: Option<String>,
    slug: String,
    name: String,
    description: Option<String>,
}

#[derive(Deserialize)]
pub struct UpdatePermissionRequest {
    name: Option<String>,
    description: Option<String>,
    is_active: Option<bool>,
}

#[web::post("/permissions")]
pub async fn create_permission(
    db: web::types::State<Arc<crate::core::infrastructure::database::Database>>,
    audit: web::types::State<Arc<AuditService>>,
    body: web::types::Json<CreatePermissionRequest>,
) -> impl web::Responder {
    let result = sqlx::query(
        "INSERT INTO sys_permissions (code, slug, name, description) VALUES ($1, $2, $3, $4) RETURNING id"
    )
    .bind(&body.code)
    .bind(&body.slug)
    .bind(&body.name)
    .bind(&body.description)
    .fetch_one(&db.pool)
    .await;

    match result {
        Ok(row) => {
            let id: sqlx::types::Uuid = row.get("id");
            let _ = audit.log("SuperAdmin", "PERMISSION_CREATED", Some(&body.slug), "SUCCESS", None).await;
            web::HttpResponse::Created().json(&serde_json::json!({ "id": id, "success": true }))
        },
        Err(e) => {
            let msg = e.to_string();
            if msg.contains("unique constraint") {
                web::HttpResponse::Conflict().json(&serde_json::json!({ "error": "Permission slug or code already exists" }))
            } else {
                web::HttpResponse::InternalServerError().json(&serde_json::json!({ "error": msg }))
            }
        }
    }
}

#[web::patch("/permissions/{id}")]
pub async fn update_permission(
    db: web::types::State<Arc<crate::core::infrastructure::database::Database>>,
    audit: web::types::State<Arc<AuditService>>,
    path: web::types::Path<String>,
    body: web::types::Json<UpdatePermissionRequest>,
) -> impl web::Responder {
    let id_uuid = match sqlx::types::Uuid::parse_str(&path) {
        Ok(u) => u,
        Err(_) => return web::HttpResponse::BadRequest().json(&serde_json::json!({ "error": "Invalid ID format" })),
    };

    let result = sqlx::query(
        "UPDATE sys_permissions SET 
         name = COALESCE($1, name), 
         description = COALESCE($2, description),
         is_active = COALESCE($3, is_active),
         updated_at = NOW()
         WHERE id = $4 AND deleted_at IS NULL"
    )
    .bind(&body.name)
    .bind(&body.description)
    .bind(body.is_active)
    .bind(id_uuid)
    .execute(&db.pool)
    .await;

    match result {
        Ok(res) => {
            if res.rows_affected() == 0 {
                return web::HttpResponse::NotFound().json(&serde_json::json!({ "error": "Permission not found" }));
            }
            let _ = audit.log("SuperAdmin", "PERMISSION_UPDATED", Some(&path), "SUCCESS", None).await;
            web::HttpResponse::Ok().json(&serde_json::json!({ "success": true }))
        },
        Err(e) => web::HttpResponse::InternalServerError().json(&serde_json::json!({ "error": e.to_string() }))
    }
}

#[web::delete("/permissions/{id}")]
pub async fn delete_permission(
    db: web::types::State<Arc<crate::core::infrastructure::database::Database>>,
    audit: web::types::State<Arc<AuditService>>,
    path: web::types::Path<String>,
) -> impl web::Responder {
    let id_uuid = match sqlx::types::Uuid::parse_str(&path) {
        Ok(u) => u,
        Err(_) => return web::HttpResponse::BadRequest().json(&serde_json::json!({ "error": "Invalid ID format" })),
    };

    let result = sqlx::query("UPDATE sys_permissions SET deleted_at = NOW() WHERE id = $1 AND deleted_at IS NULL")
        .bind(id_uuid)
        .execute(&db.pool)
        .await;

    match result {
        Ok(res) => {
            if res.rows_affected() == 0 {
                return web::HttpResponse::NotFound().json(&serde_json::json!({ "error": "Permission not found or already deleted" }));
            }
            let _ = audit.log("SuperAdmin", "PERMISSION_DELETED", Some(&path), "SUCCESS", None).await;
            web::HttpResponse::Ok().json(&serde_json::json!({ "success": true }))
        },
        Err(e) => web::HttpResponse::InternalServerError().json(&serde_json::json!({ "error": e.to_string() }))
    }
}

// === CRUD: Roles ===

#[derive(Deserialize)]
pub struct CreateRoleRequest {
    code: Option<String>,
    slug: String,
    name: String,
    sort_order: Option<i32>,
    max_members: Option<i32>,
}

#[derive(Deserialize)]
pub struct UpdateRoleRequest {
    name: Option<String>,
    is_active: Option<bool>,
    max_members: Option<i32>,
}

#[web::post("/roles")]
pub async fn create_role(
    db: web::types::State<Arc<crate::core::infrastructure::database::Database>>,
    audit: web::types::State<Arc<AuditService>>,
    body: web::types::Json<CreateRoleRequest>,
) -> impl web::Responder {
    let result = sqlx::query(
        "INSERT INTO sys_roles (code, slug, name, sort_order, max_members) VALUES ($1, $2, $3, $4, $5) RETURNING id"
    )
    .bind(&body.code)
    .bind(&body.slug)
    .bind(&body.name)
    .bind(body.sort_order.unwrap_or(0))
    .bind(body.max_members)
    .fetch_one(&db.pool)
    .await;

    match result {
        Ok(row) => {
            let id: sqlx::types::Uuid = row.get("id");
            let _ = audit.log("SuperAdmin", "ROLE_CREATED", Some(&body.slug), "SUCCESS", None).await;
            web::HttpResponse::Created().json(&serde_json::json!({ "id": id, "success": true }))
        },
        Err(e) => {
            let msg = e.to_string();
            if msg.contains("unique constraint") {
                web::HttpResponse::Conflict().json(&serde_json::json!({ "error": "Role slug or code already exists" }))
            } else {
                web::HttpResponse::InternalServerError().json(&serde_json::json!({ "error": msg }))
            }
        }
    }
}

#[web::patch("/roles/{id}")]
pub async fn update_role(
    db: web::types::State<Arc<crate::core::infrastructure::database::Database>>,
    audit: web::types::State<Arc<AuditService>>,
    path: web::types::Path<String>,
    body: web::types::Json<UpdateRoleRequest>,
) -> impl web::Responder {
    let id_uuid = match sqlx::types::Uuid::parse_str(&path) {
        Ok(u) => u,
        Err(_) => return web::HttpResponse::BadRequest().json(&serde_json::json!({ "error": "Invalid ID format" })),
    };

    let result = sqlx::query(
        "UPDATE sys_roles SET 
         name = COALESCE($1, name), 
         is_active = COALESCE($2, is_active),
         max_members = COALESCE($3, max_members),
         updated_at = NOW()
         WHERE id = $4 AND deleted_at IS NULL"
    )
    .bind(&body.name)
    .bind(body.is_active)
    .bind(body.max_members)
    .bind(id_uuid)
    .execute(&db.pool)
    .await;

    match result {
        Ok(res) => {
            if res.rows_affected() == 0 {
                return web::HttpResponse::NotFound().json(&serde_json::json!({ "error": "Role not found" }));
            }
            let _ = audit.log("SuperAdmin", "ROLE_UPDATED", Some(&path), "SUCCESS", None).await;
            web::HttpResponse::Ok().json(&serde_json::json!({ "success": true }))
        },
        Err(e) => web::HttpResponse::InternalServerError().json(&serde_json::json!({ "error": e.to_string() }))
    }
}

#[web::delete("/roles/{id}")]
pub async fn delete_role(
    db: web::types::State<Arc<crate::core::infrastructure::database::Database>>,
    audit: web::types::State<Arc<AuditService>>,
    path: web::types::Path<String>,
) -> impl web::Responder {
    let id_uuid = match sqlx::types::Uuid::parse_str(&path) {
        Ok(u) => u,
        Err(_) => return web::HttpResponse::BadRequest().json(&serde_json::json!({ "error": "Invalid ID format" })),
    };
    
    // Check if system role?
    let role = sqlx::query("SELECT slug FROM sys_roles WHERE id = $1")
        .bind(id_uuid)
        .fetch_optional(&db.pool)
        .await;

    if let Ok(Some(row)) = role {
        let slug: String = row.get("slug");
        if slug == "superadmin" {
            return web::HttpResponse::Forbidden().json(&serde_json::json!({ "error": "Cannot delete SuperAdmin role" }));
        }
    }

    let result = sqlx::query("UPDATE sys_roles SET deleted_at = NOW() WHERE id = $1 AND deleted_at IS NULL")
        .bind(id_uuid)
        .execute(&db.pool)
        .await;

    match result {
        Ok(res) => {
            if res.rows_affected() == 0 {
                return web::HttpResponse::NotFound().json(&serde_json::json!({ "error": "Role not found or already deleted" }));
            }
            let _ = audit.log("SuperAdmin", "ROLE_DELETED", Some(&path), "SUCCESS", None).await;
            web::HttpResponse::Ok().json(&serde_json::json!({ "success": true }))
        },
        Err(e) => web::HttpResponse::InternalServerError().json(&serde_json::json!({ "error": e.to_string() }))
    }
}

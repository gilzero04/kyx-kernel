use ntex::web;
use std::sync::Arc;
use crate::modules::system::application::services::rbac::RbacService;
use crate::modules::system::interface::http::dto::rbac::{
    CreateRoleRequest, UpdateRoleRequest, 
    CreatePermissionRequest, UpdatePermissionRequest
};
use crate::modules::system::domain::rbac::{
    CreateRoleCmd, UpdateRoleCmd, 
    CreatePermissionCmd, UpdatePermissionCmd
};
use crate::core::infrastructure::audit::AuditService;

// === Roles ===

/// List all roles (Admin)
#[utoipa::path(
    get,
    path = "/api/v1/admin/roles",
    responses(
        (status = 200, description = "List of roles", body = Vec<Role>)
    ),
    tag = "rbac",
    security(
        ("bearer_auth" = [])
    )
)]
pub async fn list_roles(
    service: web::types::State<Arc<RbacService>>,
) -> impl web::Responder {
    match service.list_roles().await {
        Ok(roles) => web::HttpResponse::Ok().json(&serde_json::json!({ "data": roles })),
        Err(e) => web::HttpResponse::InternalServerError().json(&serde_json::json!({ "error": e.message }))
    }
}

/// Create a new role (Admin)
#[utoipa::path(
    post,
    path = "/api/v1/admin/roles",
    request_body = CreateRoleRequest,
    responses(
        (status = 201, description = "Role created successfully", body = Role)
    ),
    tag = "rbac",
    security(
        ("bearer_auth" = [])
    )
)]
pub async fn create_role(
    service: web::types::State<Arc<RbacService>>,
    audit: web::types::State<Arc<AuditService>>,
    body: web::types::Json<CreateRoleRequest>,
) -> impl web::Responder {
    let cmd = CreateRoleCmd {
        code: None, // Or from body if added later
        slug: body.name.to_lowercase().replace(" ", "-"), // Simple slug generation
        name: body.name.clone(),
        description: body.description.clone(),
    };

    match service.create_role(cmd).await {
        Ok(role) => {
            let _ = audit.log("SuperAdmin", "ROLE_CREATED", Some(&role.name), "SUCCESS", None).await;
            web::HttpResponse::Created().json(&role)
        },
        Err(e) => web::HttpResponse::InternalServerError().json(&serde_json::json!({ "error": e.message }))
    }
}

/// Update a role (Admin)
#[utoipa::path(
    patch,
    path = "/api/v1/admin/roles/{id}",
    request_body = UpdateRoleRequest,
    responses(
        (status = 200, description = "Role updated successfully", body = Role),
        (status = 404, description = "Role not found")
    ),
    tag = "rbac",
    params(
        ("id" = String, Path, description = "Role ID")
    ),
    security(
        ("bearer_auth" = [])
    )
)]
pub async fn update_role(
    service: web::types::State<Arc<RbacService>>,
    audit: web::types::State<Arc<AuditService>>,
    path: web::types::Path<String>,
    body: web::types::Json<UpdateRoleRequest>,
) -> impl web::Responder {
    let id_uuid = match sqlx::types::Uuid::parse_str(&path) {
        Ok(u) => u,
        Err(_) => return web::HttpResponse::BadRequest().json(&serde_json::json!({ "error": "Invalid ID format" })),
    };

    let cmd = UpdateRoleCmd {
        name: body.name.clone(),
        description: body.description.clone(),
        is_active: None, // Or from body
    };

    match service.update_role(id_uuid, cmd).await {
        Ok(role) => {
            let _ = audit.log("SuperAdmin", "ROLE_UPDATED", Some(&path), "SUCCESS", None).await;
            web::HttpResponse::Ok().json(&role)
        },
        Err(e) => {
             let mut status = if e.code == 404 { web::HttpResponse::NotFound() } else { web::HttpResponse::InternalServerError() };
             status.json(&serde_json::json!({ "error": e.message }))
        }
    }
}

/// Delete a role (Admin)
#[utoipa::path(
    delete,
    path = "/api/v1/admin/roles/{id}",
    responses(
        (status = 200, description = "Role deleted successfully"),
        (status = 404, description = "Role not found")
    ),
    tag = "rbac",
    params(
        ("id" = String, Path, description = "Role ID")
    ),
    security(
        ("bearer_auth" = [])
    )
)]
pub async fn delete_role(
    service: web::types::State<Arc<RbacService>>,
    audit: web::types::State<Arc<AuditService>>,
    path: web::types::Path<String>,
) -> impl web::Responder {
    let id_uuid = match sqlx::types::Uuid::parse_str(&path) {
        Ok(u) => u,
        Err(_) => return web::HttpResponse::BadRequest().json(&serde_json::json!({ "error": "Invalid ID format" })),
    };

    match service.delete_role(id_uuid).await {
        Ok(_) => {
            let _ = audit.log("SuperAdmin", "ROLE_DELETED", Some(&path), "SUCCESS", None).await;
            web::HttpResponse::Ok().json(&serde_json::json!({ "success": true }))
        },
        Err(e) => {
             let mut status = if e.code == 404 { web::HttpResponse::NotFound() } else { web::HttpResponse::InternalServerError() };
             status.json(&serde_json::json!({ "error": e.message }))
        }
    }
}

// === Permissions ===

/// List all permissions (Admin)
#[utoipa::path(
    get,
    path = "/api/v1/admin/permissions",
    responses(
        (status = 200, description = "List of permissions", body = Vec<Permission>)
    ),
    tag = "rbac",
    security(
        ("bearer_auth" = [])
    )
)]
pub async fn list_permissions(
    service: web::types::State<Arc<RbacService>>,
) -> impl web::Responder {
    match service.list_permissions().await {
        Ok(perms) => web::HttpResponse::Ok().json(&serde_json::json!({ "data": perms })),
        Err(e) => web::HttpResponse::InternalServerError().json(&serde_json::json!({ "error": e.message }))
    }
}

/// Create a new permission (Admin)
#[utoipa::path(
    post,
    path = "/api/v1/admin/permissions",
    request_body = CreatePermissionRequest,
    responses(
        (status = 201, description = "Permission created successfully", body = Permission)
    ),
    tag = "rbac",
    security(
        ("bearer_auth" = [])
    )
)]
pub async fn create_permission(
    service: web::types::State<Arc<RbacService>>,
    audit: web::types::State<Arc<AuditService>>,
    body: web::types::Json<CreatePermissionRequest>,
) -> impl web::Responder {
    let cmd = CreatePermissionCmd {
        code: None,
        slug: body.name.to_lowercase().replace(" ", "-"),
        name: body.name.clone(),
        description: body.description.clone(),
    };

    match service.create_permission(cmd).await {
        Ok(perm) => {
            let _ = audit.log("SuperAdmin", "PERMISSION_CREATED", Some(&perm.name), "SUCCESS", None).await;
            web::HttpResponse::Created().json(&perm)
        },
        Err(e) => web::HttpResponse::InternalServerError().json(&serde_json::json!({ "error": e.message }))
    }
}

/// Update a permission (Admin)
#[utoipa::path(
    patch,
    path = "/api/v1/admin/permissions/{id}",
    request_body = UpdatePermissionRequest,
    responses(
        (status = 200, description = "Permission updated successfully", body = Permission),
        (status = 404, description = "Permission not found")
    ),
    tag = "rbac",
    params(
        ("id" = String, Path, description = "Permission ID")
    ),
    security(
        ("bearer_auth" = [])
    )
)]
pub async fn update_permission(
    service: web::types::State<Arc<RbacService>>,
    audit: web::types::State<Arc<AuditService>>,
    path: web::types::Path<String>,
    body: web::types::Json<UpdatePermissionRequest>,
) -> impl web::Responder {
    let id_uuid = match sqlx::types::Uuid::parse_str(&path) {
        Ok(u) => u,
        Err(_) => return web::HttpResponse::BadRequest().json(&serde_json::json!({ "error": "Invalid ID format" })),
    };

    let cmd = UpdatePermissionCmd {
        name: body.name.clone(),
        description: body.description.clone(),
        is_active: None,
    };

    match service.update_permission(id_uuid, cmd).await {
        Ok(perm) => {
            let _ = audit.log("SuperAdmin", "PERMISSION_UPDATED", Some(&path), "SUCCESS", None).await;
            web::HttpResponse::Ok().json(&perm)
        },
        Err(e) => {
             let mut status = if e.code == 404 { web::HttpResponse::NotFound() } else { web::HttpResponse::InternalServerError() };
             status.json(&serde_json::json!({ "error": e.message }))
        }
    }
}

/// Delete a permission (Admin)
#[utoipa::path(
    delete,
    path = "/api/v1/admin/permissions/{id}",
    responses(
        (status = 200, description = "Permission deleted successfully"),
        (status = 404, description = "Permission not found")
    ),
    tag = "rbac",
    params(
        ("id" = String, Path, description = "Permission ID")
    ),
    security(
        ("bearer_auth" = [])
    )
)]
pub async fn delete_permission(
    service: web::types::State<Arc<RbacService>>,
    audit: web::types::State<Arc<AuditService>>,
    path: web::types::Path<String>,
) -> impl web::Responder {
    let id_uuid = match sqlx::types::Uuid::parse_str(&path) {
        Ok(u) => u,
        Err(_) => return web::HttpResponse::BadRequest().json(&serde_json::json!({ "error": "Invalid ID format" })),
    };

    match service.delete_permission(id_uuid).await {
        Ok(_) => {
            let _ = audit.log("SuperAdmin", "PERMISSION_DELETED", Some(&path), "SUCCESS", None).await;
            web::HttpResponse::Ok().json(&serde_json::json!({ "success": true }))
        },
        Err(e) => {
             let mut status = if e.code == 404 { web::HttpResponse::NotFound() } else { web::HttpResponse::InternalServerError() };
             status.json(&serde_json::json!({ "error": e.message }))
        }
    }
}

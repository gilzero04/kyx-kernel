use crate::core::infrastructure::audit::AuditService;
use crate::core::utils::jwt::Claims;
use crate::core::utils::response::ApiResponse;
use crate::modules::system::application::services::rbac::RbacService;
use crate::modules::system::application::services::tenant::TenantService;
use crate::modules::system::domain::rbac::{
    CreatePermissionCmd, CreateRoleCmd, UpdatePermissionCmd, UpdateRoleCmd,
};
use crate::modules::system::interface::http::dto::rbac::{
    CreatePermissionRequest, CreateRoleRequest, RoleFilter, UpdatePermissionRequest,
    UpdateRolePermissionsRequest, UpdateRoleRequest,
};
use ntex::web;
use std::sync::Arc;
use uuid::Uuid;

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
    tenant_service: web::types::State<Arc<TenantService>>,
    claims: Claims,
    query: web::types::Query<RoleFilter>,
) -> impl web::Responder {
    let actor_tenant_id = Some(claims.tenant_id);

    // Identify if actor is System Owner
    let is_owner = if let Ok(owner_id) = tenant_service.get_owner_id().await {
        actor_tenant_id == Some(owner_id)
    } else {
        false
    };

    // If actor is system owner, treat as None (allowing broad filter)
    let effective_actor_tid = if is_owner { None } else { actor_tenant_id };

    // Parse requested filter
    let mut target_tenant_id = None;
    let mut show_all = false;

    if let Some(ref tid_str) = query.tenant_id {
        match tid_str.as_str() {
            "all" => show_all = true,
            "global" => target_tenant_id = None,
            uuid_str => {
                if let Ok(uid) = Uuid::parse_str(uuid_str) {
                    target_tenant_id = Some(uid);
                }
            }
        }
    }

    match service
        .list_roles(target_tenant_id, effective_actor_tid, show_all)
        .await
    {
        Ok(roles) => {
            let response =
                ApiResponse::ok(serde_json::json!({ "roles": roles }), "Roles retrieved");
            web::HttpResponse::Ok().json(&response)
        }
        Err(e) => {
            let response = ApiResponse::<()>::internal_error(&e.message);
            web::HttpResponse::InternalServerError().json(&response)
        }
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
    tenant_service: web::types::State<Arc<TenantService>>,
    audit: web::types::State<Arc<AuditService>>,
    claims: Claims,
    body: web::types::Json<CreateRoleRequest>,
) -> impl web::Responder {
    let mut actor_tenant_id = Some(claims.tenant_id);

    // If actor is system owner, treat as None
    if let Ok(owner_id) = tenant_service.get_owner_id().await
        && let Some(tid) = actor_tenant_id
            && tid == owner_id {
                actor_tenant_id = None;
            }

    let cmd = CreateRoleCmd {
        tenant_id: None, // Will be set by service based on actor_tenant_id
        code: body.code.clone(),
        slug: body
            .slug
            .clone()
            .unwrap_or_else(|| body.name.to_lowercase().replace(" ", "-")),
        name: body.name.clone(),
        description: body.description.clone(),
        is_active: body.is_active,
    };

    match service.create_role(cmd, actor_tenant_id).await {
        Ok(role) => {
            let _ = audit
                .log(
                    "SuperAdmin",
                    "ROLE_CREATED",
                    Some(&role.name),
                    "SUCCESS",
                    None,
                )
                .await;
            let response = ApiResponse::created(role, "Role created");
            web::HttpResponse::Created().json(&response)
        }
        Err(e) => {
            let response = ApiResponse::<()>::internal_error(&e.message);
            web::HttpResponse::InternalServerError().json(&response)
        }
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
    tenant_service: web::types::State<Arc<TenantService>>,
    audit: web::types::State<Arc<AuditService>>,
    claims: Claims,
    path: web::types::Path<String>,
    body: web::types::Json<UpdateRoleRequest>,
) -> impl web::Responder {
    let mut actor_tenant_id = Some(claims.tenant_id);

    if let Ok(owner_id) = tenant_service.get_owner_id().await
        && let Some(tid) = actor_tenant_id
            && tid == owner_id {
                actor_tenant_id = None;
            }

    let id_uuid = match sqlx::types::Uuid::parse_str(&path) {
        Ok(u) => u,
        Err(_) => {
            let response = ApiResponse::<()>::bad_request("Invalid ID format");
            return web::HttpResponse::BadRequest().json(&response);
        }
    };

    let cmd = UpdateRoleCmd {
        name: body.name.clone(),
        code: body.code.clone(),
        description: body.description.clone(),
        is_active: body.is_active,
    };

    match service.update_role(id_uuid, cmd, actor_tenant_id).await {
        Ok(role) => {
            let _ = audit
                .log("SuperAdmin", "ROLE_UPDATED", Some(&path), "SUCCESS", None)
                .await;
            let response = ApiResponse::ok(role, "Role updated");
            web::HttpResponse::Ok().json(&response)
        }
        Err(e) => {
            let response = if e.code == 404 {
                ApiResponse::<()>::not_found(&e.message)
            } else {
                ApiResponse::<()>::internal_error(&e.message)
            };
            let mut status = if e.code == 404 {
                web::HttpResponse::NotFound()
            } else {
                web::HttpResponse::InternalServerError()
            };
            status.json(&response)
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
    tenant_service: web::types::State<Arc<TenantService>>,
    audit: web::types::State<Arc<AuditService>>,
    claims: Claims,
    path: web::types::Path<String>,
) -> impl web::Responder {
    let mut actor_tenant_id = Some(claims.tenant_id);

    if let Ok(owner_id) = tenant_service.get_owner_id().await
        && let Some(tid) = actor_tenant_id
            && tid == owner_id {
                actor_tenant_id = None;
            }

    let id_uuid = match sqlx::types::Uuid::parse_str(&path) {
        Ok(u) => u,
        Err(_) => {
            let response = ApiResponse::<()>::bad_request("Invalid ID format");
            return web::HttpResponse::BadRequest().json(&response);
        }
    };

    match service.delete_role(id_uuid, actor_tenant_id).await {
        Ok(_) => {
            let _ = audit
                .log("SuperAdmin", "ROLE_DELETED", Some(&path), "SUCCESS", None)
                .await;
            let response = ApiResponse::ok(serde_json::json!({ "deleted": true }), "Role deleted");
            web::HttpResponse::Ok().json(&response)
        }
        Err(e) => {
            let response = if e.code == 404 {
                ApiResponse::<()>::not_found(&e.message)
            } else {
                ApiResponse::<()>::internal_error(&e.message)
            };
            let mut status = if e.code == 404 {
                web::HttpResponse::NotFound()
            } else {
                web::HttpResponse::InternalServerError()
            };
            status.json(&response)
        }
    }
}

/// Get role permissions (Admin)
#[utoipa::path(
    get,
    path = "/api/v1/admin/roles/{id}/permissions",
    responses(
        (status = 200, description = "List of role permissions", body = Vec<Permission>),
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
pub async fn get_role_permissions(
    service: web::types::State<Arc<RbacService>>,
    tenant_service: web::types::State<Arc<TenantService>>,
    claims: Claims,
    path: web::types::Path<String>,
) -> impl web::Responder {
    let mut actor_tenant_id = Some(claims.tenant_id);

    if let Ok(owner_id) = tenant_service.get_owner_id().await
        && let Some(tid) = actor_tenant_id
            && tid == owner_id {
                actor_tenant_id = None;
            }

    let id_uuid = match sqlx::types::Uuid::parse_str(&path) {
        Ok(u) => u,
        Err(_) => {
            let response = ApiResponse::<()>::bad_request("Invalid ID format");
            return web::HttpResponse::BadRequest().json(&response);
        }
    };

    match service.get_role_permissions(id_uuid, actor_tenant_id).await {
        Ok(perms) => {
            let response = ApiResponse::ok(
                serde_json::json!({ "permissions": perms }),
                "Role permissions retrieved",
            );
            web::HttpResponse::Ok().json(&response)
        }
        Err(e) => {
            let response = if e.code == 404 {
                ApiResponse::<()>::not_found(&e.message)
            } else {
                ApiResponse::<()>::internal_error(&e.message)
            };
            let mut status = if e.code == 404 {
                web::HttpResponse::NotFound()
            } else {
                web::HttpResponse::InternalServerError()
            };
            status.json(&response)
        }
    }
}

/// Update role permissions (Admin)
#[utoipa::path(
    post,
    path = "/api/v1/admin/roles/{id}/permissions",
    request_body = UpdateRolePermissionsRequest,
    responses(
        (status = 200, description = "Role permissions updated successfully"),
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
pub async fn update_role_permissions(
    service: web::types::State<Arc<RbacService>>,
    tenant_service: web::types::State<Arc<TenantService>>,
    audit: web::types::State<Arc<AuditService>>,
    claims: Claims,
    path: web::types::Path<String>,
    body: web::types::Json<UpdateRolePermissionsRequest>,
) -> impl web::Responder {
    let mut actor_tenant_id = Some(claims.tenant_id);

    if let Ok(owner_id) = tenant_service.get_owner_id().await
        && let Some(tid) = actor_tenant_id
            && tid == owner_id {
                actor_tenant_id = None;
            }

    let id_uuid = match sqlx::types::Uuid::parse_str(&path) {
        Ok(u) => u,
        Err(_) => {
            let response = ApiResponse::<()>::bad_request("Invalid ID format");
            return web::HttpResponse::BadRequest().json(&response);
        }
    };

    match service
        .update_role_permissions(id_uuid, body.permission_ids.clone(), actor_tenant_id)
        .await
    {
        Ok(_) => {
            let _ = audit
                .log(
                    "SuperAdmin",
                    "ROLE_PERMISSIONS_UPDATED",
                    Some(&path),
                    "SUCCESS",
                    None,
                )
                .await;
            let response = ApiResponse::ok(
                serde_json::json!({ "updated": true }),
                "Role permissions updated",
            );
            web::HttpResponse::Ok().json(&response)
        }
        Err(e) => {
            let response = if e.code == 404 {
                ApiResponse::<()>::not_found(&e.message)
            } else {
                ApiResponse::<()>::internal_error(&e.message)
            };
            let mut status = if e.code == 404 {
                web::HttpResponse::NotFound()
            } else {
                web::HttpResponse::InternalServerError()
            };
            status.json(&response)
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
    tenant_service: web::types::State<Arc<TenantService>>,
    claims: Claims,
) -> impl web::Responder {
    let mut actor_tenant_id = Some(claims.tenant_id);

    // If actor is system owner, treat as None for "unfiltered" access
    if let Ok(owner_id) = tenant_service.get_owner_id().await
        && let Some(tid) = actor_tenant_id
            && tid == owner_id {
                actor_tenant_id = None;
            }

    match service.list_permissions(actor_tenant_id).await {
        Ok(perms) => {
            let response = ApiResponse::ok(
                serde_json::json!({ "permissions": perms }),
                "Permissions retrieved",
            );
            web::HttpResponse::Ok().json(&response)
        }
        Err(e) => {
            let response = ApiResponse::<()>::internal_error(&e.message);
            web::HttpResponse::InternalServerError().json(&response)
        }
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
    tenant_service: web::types::State<Arc<TenantService>>,
    audit: web::types::State<Arc<AuditService>>,
    claims: Claims,
    body: web::types::Json<CreatePermissionRequest>,
) -> impl web::Responder {
    let mut actor_tenant_id = Some(claims.tenant_id);

    // If actor is system owner, treat as None
    if let Ok(owner_id) = tenant_service.get_owner_id().await
        && let Some(tid) = actor_tenant_id
            && tid == owner_id {
                actor_tenant_id = None;
            }

    let cmd = CreatePermissionCmd {
        code: body.code.clone(),
        slug: body
            .slug
            .clone()
            .unwrap_or_else(|| body.name.to_lowercase().replace(" ", "-")),
        name: body.name.clone(),
        description: body.description.clone(),
        is_active: body.is_active,
    };

    match service.create_permission(cmd, actor_tenant_id).await {
        Ok(perm) => {
            let _ = audit
                .log(
                    "SuperAdmin",
                    "PERMISSION_CREATED",
                    Some(&perm.name),
                    "SUCCESS",
                    None,
                )
                .await;
            let response = ApiResponse::created(perm, "Permission created");
            web::HttpResponse::Created().json(&response)
        }
        Err(e) => {
            let response = if e.code == 403 {
                ApiResponse::<()>::forbidden(&e.message)
            } else {
                ApiResponse::<()>::internal_error(&e.message)
            };
            let mut status = if e.code == 403 {
                web::HttpResponse::Forbidden()
            } else {
                web::HttpResponse::InternalServerError()
            };
            status.json(&response)
        }
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
    tenant_service: web::types::State<Arc<TenantService>>,
    audit: web::types::State<Arc<AuditService>>,
    claims: Claims,
    path: web::types::Path<String>,
    body: web::types::Json<UpdatePermissionRequest>,
) -> impl web::Responder {
    let mut actor_tenant_id = Some(claims.tenant_id);

    let id_uuid = match sqlx::types::Uuid::parse_str(&path) {
        Ok(u) => u,
        Err(_) => {
            let response = ApiResponse::<()>::bad_request("Invalid ID format");
            return web::HttpResponse::BadRequest().json(&response);
        }
    };

    // If actor is system owner, treat as None
    if let Ok(owner_id) = tenant_service.get_owner_id().await
        && claims.tenant_id == owner_id {
            actor_tenant_id = None;
        }

    let cmd = UpdatePermissionCmd {
        name: body.name.clone(),
        code: body.code.clone(),
        description: body.description.clone(),
        is_active: body.is_active,
    };

    match service
        .update_permission(id_uuid, cmd, actor_tenant_id)
        .await
    {
        Ok(perm) => {
            let _ = audit
                .log(
                    "SuperAdmin",
                    "PERMISSION_UPDATED",
                    Some(&path),
                    "SUCCESS",
                    None,
                )
                .await;
            let response = ApiResponse::ok(perm, "Permission updated");
            web::HttpResponse::Ok().json(&response)
        }
        Err(e) => {
            let response = if e.code == 404 {
                ApiResponse::<()>::not_found(&e.message)
            } else if e.code == 403 {
                ApiResponse::<()>::forbidden(&e.message)
            } else {
                ApiResponse::<()>::internal_error(&e.message)
            };
            let mut status = if e.code == 404 {
                web::HttpResponse::NotFound()
            } else if e.code == 403 {
                web::HttpResponse::Forbidden()
            } else {
                web::HttpResponse::InternalServerError()
            };
            status.json(&response)
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
    tenant_service: web::types::State<Arc<TenantService>>,
    audit: web::types::State<Arc<AuditService>>,
    claims: Claims,
    path: web::types::Path<String>,
) -> impl web::Responder {
    let mut actor_tenant_id = Some(claims.tenant_id);

    let id_uuid = match sqlx::types::Uuid::parse_str(&path) {
        Ok(u) => u,
        Err(_) => {
            let response = ApiResponse::<()>::bad_request("Invalid ID format");
            return web::HttpResponse::BadRequest().json(&response);
        }
    };

    // If actor is system owner, treat as None
    if let Ok(owner_id) = tenant_service.get_owner_id().await
        && claims.tenant_id == owner_id {
            actor_tenant_id = None;
        }

    match service.delete_permission(id_uuid, actor_tenant_id).await {
        Ok(_) => {
            let _ = audit
                .log(
                    "SuperAdmin",
                    "PERMISSION_DELETED",
                    Some(&path),
                    "SUCCESS",
                    None,
                )
                .await;
            let response =
                ApiResponse::ok(serde_json::json!({ "deleted": true }), "Permission deleted");
            web::HttpResponse::Ok().json(&response)
        }
        Err(e) => {
            let response = if e.code == 404 {
                ApiResponse::<()>::not_found(&e.message)
            } else if e.code == 403 {
                ApiResponse::<()>::forbidden(&e.message)
            } else {
                ApiResponse::<()>::internal_error(&e.message)
            };
            let mut status = if e.code == 404 {
                web::HttpResponse::NotFound()
            } else if e.code == 403 {
                web::HttpResponse::Forbidden()
            } else {
                web::HttpResponse::InternalServerError()
            };
            status.json(&response)
        }
    }
}

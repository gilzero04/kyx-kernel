use ntex::web;
use std::sync::Arc;
use crate::modules::system::application::services::user::UserAdminService;
use crate::modules::system::application::services::tenant::TenantService;
use crate::modules::system::interface::http::dto::user::{UsersQuery, UpdateUserRequest, AdminResetPasswordRequest};
use crate::modules::system::domain::user::UserFilter;
use crate::core::infrastructure::audit::AuditService;
use crate::core::utils::jwt::Claims;

/// List all users (Admin)
#[utoipa::path(
    get,
    path = "/api/v1/admin/users",
    params(
        UsersQuery
    ),
    responses(
        (status = 200, description = "List of users", body = PaginatedUsers)
    ),
    tag = "users",
    security(
        ("bearer_auth" = [])
    )
)]
pub async fn list_users(
    service: web::types::State<Arc<UserAdminService>>,
    tenant_service: web::types::State<Arc<TenantService>>,
    claims: Claims,
    query: web::types::Query<UsersQuery>,
) -> impl web::Responder {
    let mut actor_tenant_id = Some(claims.tenant_id);

    // Bypass isolation ONLY for System Admins (Admin/SuperAdmin in Owner Tenant)
    let role = claims.role.to_lowercase();
    if role == "admin" || role == "superadmin" {
        if let Ok(owner_id) = tenant_service.get_owner_id().await {
            if claims.tenant_id == owner_id {
                actor_tenant_id = None;
            }
        }
    }

    let filter = UserFilter {
        page: query.page.unwrap_or(1),
        limit: query.limit.unwrap_or(10),
        sort: query.sort.clone(),
        search: query.search.clone(),
        tenant_id: None,
        actor_tenant_id: None,
    };

    match service.list_users(filter, actor_tenant_id).await {
        Ok(data) => web::HttpResponse::Ok().json(&data),
        Err(e) => web::HttpResponse::InternalServerError().json(&serde_json::json!({
            "error": e.message
        }))
    }
}

/// Update a user (Admin)
#[utoipa::path(
    patch,
    path = "/api/v1/admin/users/{id}",
    request_body = UpdateUserRequest,
    responses(
        (status = 200, description = "User updated successfully"),
        (status = 404, description = "User not found")
    ),
    tag = "users",
    params(
        ("id" = String, Path, description = "User ID")
    ),
    security(
        ("bearer_auth" = [])
    )
)]
pub async fn update_user(
    service: web::types::State<Arc<UserAdminService>>,
    audit: web::types::State<Arc<AuditService>>,
    tenant_service: web::types::State<Arc<TenantService>>,
    claims: Claims,
    path: web::types::Path<String>,
    body: web::types::Json<UpdateUserRequest>,
) -> impl web::Responder {
    let id_uuid = match sqlx::types::Uuid::parse_str(&path) {
        Ok(u) => u,
        Err(_) => return web::HttpResponse::BadRequest().json(&serde_json::json!({ "error": "Invalid ID format" })),
    };

    let mut actor_tenant_id = Some(claims.tenant_id);

    // Bypass isolation ONLY for System Admins (Admin/SuperAdmin in Owner Tenant)
    let role = claims.role.to_lowercase();
    if role == "admin" || role == "superadmin" {
        if let Ok(owner_id) = tenant_service.get_owner_id().await {
            if claims.tenant_id == owner_id {
                actor_tenant_id = None;
            }
        }
    }

    match service.update_user(id_uuid, body.full_name.clone(), body.is_active, body.role_slug.clone(), body.tenant_id, actor_tenant_id).await {
        Ok(_) => {
            let _ = audit.log("SuperAdmin", "USER_UPDATED", Some(&path), "SUCCESS", None).await;
            web::HttpResponse::Ok().json(&serde_json::json!({ "success": true }))
        },
        Err(e) => {
            let mut status = if e.code == 404 { web::HttpResponse::NotFound() } else { web::HttpResponse::InternalServerError() };
            status.json(&serde_json::json!({ "error": e.message }))
        }
    }
}

/// Delete a user (Admin)
#[utoipa::path(
    delete,
    path = "/api/v1/admin/users/{id}",
    responses(
        (status = 200, description = "User deleted successfully"),
        (status = 404, description = "User not found")
    ),
    tag = "users",
    params(
        ("id" = String, Path, description = "User ID")
    ),
    security(
        ("bearer_auth" = [])
    )
)]
pub async fn delete_user(
    service: web::types::State<Arc<UserAdminService>>,
    audit: web::types::State<Arc<AuditService>>,
    tenant_service: web::types::State<Arc<TenantService>>,
    claims: Claims,
    path: web::types::Path<String>,
) -> impl web::Responder {
    let id_uuid = match sqlx::types::Uuid::parse_str(&path) {
        Ok(u) => u,
        Err(_) => return web::HttpResponse::BadRequest().json(&serde_json::json!({ "error": "Invalid ID format" })),
    };

    let current_user_id = sqlx::types::Uuid::parse_str(&claims.sub).ok();
    let mut actor_tenant_id = Some(claims.tenant_id);

    // Bypass isolation ONLY for System Admins
    let role = claims.role.to_lowercase();
    if role == "admin" || role == "superadmin" {
        if let Ok(owner_id) = tenant_service.get_owner_id().await {
            if claims.tenant_id == owner_id {
                actor_tenant_id = None; // Disable isolation
            }
        }
    }

    match service.delete_user(id_uuid, current_user_id, actor_tenant_id).await {
        Ok(_) => {
            let _ = audit.log("SuperAdmin", "USER_DELETED", Some(&path), "SUCCESS", None).await;
            web::HttpResponse::Ok().json(&serde_json::json!({ "success": true }))
        },
        Err(e) => {
             let mut status = if e.code == 404 { web::HttpResponse::NotFound() } else { web::HttpResponse::BadRequest() };
             status.json(&serde_json::json!({ "error": e.message }))
        }
    }
}

/// Reset a user password (Admin)
#[utoipa::path(
    post,
    path = "/api/v1/admin/users/{id}/reset-password",
    request_body = AdminResetPasswordRequest,
    responses(
        (status = 200, description = "Password reset successfully"),
        (status = 404, description = "User not found")
    ),
    tag = "users",
    params(
        ("id" = String, Path, description = "User ID")
    ),
    security(
        ("bearer_auth" = [])
    )
)]
pub async fn reset_password(
    service: web::types::State<Arc<UserAdminService>>,
    tenant_service: web::types::State<Arc<TenantService>>,
    audit: web::types::State<Arc<AuditService>>,
    claims: Claims,
    path: web::types::Path<String>,
    body: web::types::Json<AdminResetPasswordRequest>,
) -> impl web::Responder {
    let id_uuid = match sqlx::types::Uuid::parse_str(&path) {
        Ok(u) => u,
        Err(_) => return web::HttpResponse::BadRequest().json(&serde_json::json!({ "error": "Invalid ID format" })),
    };

    let mut actor_tenant_id = Some(claims.tenant_id);

    // Bypass isolation ONLY for System Admins (Admin/SuperAdmin in Owner Tenant)
    let role = claims.role.to_lowercase();
    if role == "admin" || role == "superadmin" {
        if let Ok(owner_id) = tenant_service.get_owner_id().await {
            if claims.tenant_id == owner_id {
                actor_tenant_id = None;
            }
        }
    }

    match service.reset_password(id_uuid, body.new_password.clone(), actor_tenant_id).await {
        Ok(_) => {
            let _ = audit.log("SuperAdmin", "USER_PASSWORD_RESET", Some(&path), "SUCCESS", None).await;
            web::HttpResponse::Ok().json(&serde_json::json!({ "success": true }))
        },
        Err(e) => {
             let mut status = if e.code == 404 { web::HttpResponse::NotFound() } else { web::HttpResponse::InternalServerError() };
             status.json(&serde_json::json!({ "error": e.message }))
        }
    }
}

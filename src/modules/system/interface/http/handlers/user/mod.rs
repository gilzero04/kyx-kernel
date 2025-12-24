use ntex::web;
use std::sync::Arc;
use crate::modules::system::application::services::user::UserAdminService;
use crate::modules::system::interface::http::dto::user::{UsersQuery, UpdateUserRequest};
use crate::modules::system::domain::user::UserFilter;
use crate::core::infrastructure::audit::AuditService;
use crate::core::utils::jwt::JwtService;

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
    query: web::types::Query<UsersQuery>,
) -> impl web::Responder {
    let filter = UserFilter {
        page: query.page.unwrap_or(1),
        limit: query.limit.unwrap_or(10),
        sort: query.sort.clone(),
        search: query.search.clone(),
    };

    match service.list_users(filter).await {
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
    path: web::types::Path<String>,
    body: web::types::Json<UpdateUserRequest>,
) -> impl web::Responder {
    let id_uuid = match sqlx::types::Uuid::parse_str(&path) {
        Ok(u) => u,
        Err(_) => return web::HttpResponse::BadRequest().json(&serde_json::json!({ "error": "Invalid ID format" })),
    };

    match service.update_user(id_uuid, body.full_name.clone(), body.is_active).await {
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
    jwt_service: web::types::State<Arc<JwtService>>,
    req: web::HttpRequest,
    path: web::types::Path<String>,
) -> impl web::Responder {
    let id_uuid = match sqlx::types::Uuid::parse_str(&path) {
        Ok(u) => u,
        Err(_) => return web::HttpResponse::BadRequest().json(&serde_json::json!({ "error": "Invalid ID format" })),
    };

    // Extract current user ID
    let mut current_user_id = None;
    if let Some(header) = req.headers().get("Authorization") {
        if let Ok(val) = header.to_str() {
            if val.starts_with("Bearer ") {
                if let Ok(claims) = jwt_service.verify_token(&val[7..]) {
                     if let Ok(uid) = sqlx::types::Uuid::parse_str(&claims.sub) {
                         current_user_id = Some(uid);
                     }
                }
            }
        }
    }

    match service.delete_user(id_uuid, current_user_id).await {
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

use ntex::web;
use crate::core::infrastructure::audit::AuditService;
use serde::{Deserialize, Serialize};
use std::sync::Arc;
use sqlx::FromRow;
use chrono::{DateTime, Utc};

// === List Users ===

#[derive(Debug, Serialize, FromRow)]
pub struct UserEntry {
    id: sqlx::types::Uuid,
    email: String,
    full_name: Option<String>,
    is_active: bool,
    created_at: DateTime<Utc>,
    // Joined fields from memberships/roles
    role: Option<String>,
    tenant_name: Option<String>,
}

#[derive(Debug, Deserialize)]
pub struct UsersQuery {
    limit: Option<i64>,
    offset: Option<i64>,
    search: Option<String>,
}

#[web::get("/users")]
pub async fn list_users(
    db: web::types::State<Arc<crate::core::infrastructure::database::Database>>,
    query: web::types::Query<UsersQuery>,
) -> impl web::Responder {
    let limit = query.limit.unwrap_or(50).min(100);
    let offset = query.offset.unwrap_or(0);

    // Join with auth_memberships and sys_roles to get role info.
    // Join with auth_tenants to get tenant info.
    // For now, we list users and their primary role (assuming single tenant usage or showing first one).
    // In a multi-tenant strict view, we'd filter by current tenant from JWT.
    // But this is SuperAdmin implementation, so listing all users is fine.
    
    let sql = r#"
        SELECT 
            u.id, u.email, u.full_name, u.is_active, u.created_at,
            r.name as role,
            t.name as tenant_name
        FROM auth_users u
        LEFT JOIN auth_memberships m ON u.id = m.user_id AND m.is_active = TRUE
        LEFT JOIN sys_roles r ON m.role_id = r.id
        LEFT JOIN auth_tenants t ON m.tenant_id = t.id
        WHERE u.deleted_at IS NULL
    "#;

    // Append search if present
    let (sql, bind_search) = if let Some(s) = &query.search {
        (format!("{} AND (u.email ILIKE $1 OR u.full_name ILIKE $1) ORDER BY u.created_at DESC LIMIT $2 OFFSET $3", sql), Some(format!("%{}%", s)))
    } else {
        (format!("{} ORDER BY u.created_at DESC LIMIT $1 OFFSET $2", sql), None)
    };

    let users_result = if let Some(search) = bind_search {
        sqlx::query_as::<_, UserEntry>(&sql)
            .bind(search)
            .bind(limit)
            .bind(offset)
            .fetch_all(&db.pool)
            .await
    } else {
        sqlx::query_as::<_, UserEntry>(&sql)
            .bind(limit)
            .bind(offset)
            .fetch_all(&db.pool)
            .await
    };

    match users_result {
        Ok(entries) => web::HttpResponse::Ok().json(&entries),
        Err(e) => web::HttpResponse::InternalServerError().json(&serde_json::json!({
            "error": format!("Failed to fetch users: {}", e)
        }))
    }
}

// === Update User ===

#[derive(Deserialize)]
pub struct UpdateUserRequest {
    full_name: Option<String>,
    is_active: Option<bool>,
}

#[web::patch("/users/{id}")]
pub async fn update_user(
    db: web::types::State<Arc<crate::core::infrastructure::database::Database>>,
    audit: web::types::State<Arc<AuditService>>,
    path: web::types::Path<String>,
    body: web::types::Json<UpdateUserRequest>,
) -> impl web::Responder {
    let id_uuid = match sqlx::types::Uuid::parse_str(&path) {
        Ok(u) => u,
        Err(_) => return web::HttpResponse::BadRequest().json(&serde_json::json!({ "error": "Invalid ID format" })),
    };

    let result = sqlx::query(
        "UPDATE auth_users SET 
         full_name = COALESCE($1, full_name), 
         is_active = COALESCE($2, is_active),
         updated_at = NOW()
         WHERE id = $3 AND deleted_at IS NULL"
    )
    .bind(&body.full_name)
    .bind(body.is_active)
    .bind(id_uuid)
    .execute(&db.pool)
    .await;

    match result {
        Ok(res) => {
            if res.rows_affected() == 0 {
                return web::HttpResponse::NotFound().json(&serde_json::json!({ "error": "User not found" }));
            }
            let _ = audit.log("SuperAdmin", "USER_UPDATED", Some(&path), "SUCCESS", None).await;
            web::HttpResponse::Ok().json(&serde_json::json!({ "success": true }))
        },
        Err(e) => web::HttpResponse::InternalServerError().json(&serde_json::json!({ "error": e.to_string() }))
    }
}

// === Delete User ===

#[web::delete("/users/{id}")]
pub async fn delete_user(
    db: web::types::State<Arc<crate::core::infrastructure::database::Database>>,
    audit: web::types::State<Arc<AuditService>>,
    path: web::types::Path<String>,
) -> impl web::Responder {
    let id_uuid = match sqlx::types::Uuid::parse_str(&path) {
        Ok(u) => u,
        Err(_) => return web::HttpResponse::BadRequest().json(&serde_json::json!({ "error": "Invalid ID format" })),
    };

    // Prevent deleting self? Ideally yes, but logic is complex here without knowing current user.
    // Frontend should prevent it.
    // Also prevent deleting the last SuperAdmin?
    
    // Check if user is SuperAdmin
    let is_superadmin = sqlx::query_scalar::<_, bool>(
        "SELECT EXISTS(
            SELECT 1 FROM auth_memberships m 
            JOIN sys_roles r ON m.role_id = r.id 
            WHERE m.user_id = $1 AND r.slug = 'superadmin' AND m.is_active = TRUE
        )"
    )
    .bind(id_uuid)
    .fetch_one(&db.pool)
    .await
    .unwrap_or(false);

    if is_superadmin {
        // Count SuperAdmins
         let count_super: i64 = sqlx::query_scalar(
            "SELECT COUNT(*) FROM auth_memberships m 
             JOIN sys_roles r ON m.role_id = r.id 
             WHERE r.slug = 'superadmin' AND m.is_active = TRUE"
        )
        .fetch_one(&db.pool)
        .await
        .unwrap_or(0);
        
        if count_super <= 1 {
            return web::HttpResponse::BadRequest().json(&serde_json::json!({ 
                "error": "Cannot delete the last Super Administrator" 
            }));
        }
    }

    let result = sqlx::query("UPDATE auth_users SET deleted_at = NOW() WHERE id = $1 AND deleted_at IS NULL")
        .bind(id_uuid)
        .execute(&db.pool)
        .await;

    match result {
        Ok(res) => {
            if res.rows_affected() == 0 {
                return web::HttpResponse::NotFound().json(&serde_json::json!({ "error": "User not found or already deleted" }));
            }
            let _ = audit.log("SuperAdmin", "USER_DELETED", Some(&path), "SUCCESS", None).await;
            web::HttpResponse::Ok().json(&serde_json::json!({ "success": true }))
        },
        Err(e) => web::HttpResponse::InternalServerError().json(&serde_json::json!({ "error": e.to_string() }))
    }
}

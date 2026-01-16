use ntex::web;
use std::sync::Arc;
use serde::{Deserialize, Serialize};
use uuid::Uuid;
use crate::core::infrastructure::database::Database;
use crate::core::utils::jwt::Claims;
use crate::modules::system::infrastructure::repositories::share::{ShareRepository, CreateShareCmd, ResourceShare};

#[derive(Debug, Serialize)]
pub struct ShareResponse {
    pub success: bool,
    pub data: Option<ResourceShare>,
    pub message: Option<String>,
}

#[derive(Debug, Serialize)]
pub struct ShareListResponse {
    pub success: bool,
    pub data: Vec<ResourceShare>,
}

#[derive(Debug, Deserialize)]
pub struct CreateShareRequest {
    pub resource_type: String,
    pub resource_id: Uuid,
    pub shared_to_tenant_id: Uuid,
    pub can_reshare: Option<bool>,
}

#[derive(Debug, Serialize)]
pub struct RevokeResponse {
    pub success: bool,
    pub message: String,
    pub usage_count: Option<i32>,
}

#[derive(Debug, Deserialize)]
pub struct ForceRevokeQuery {
    pub force: Option<bool>,
}

/// GET /admin/shares - List shares created by current tenant
pub async fn list_shares(
    db: web::types::State<Arc<Database>>,
    claims: Claims,
) -> impl web::Responder {
    let tenant_id = claims.tenant_id;

    let repo = ShareRepository::new(db.get_ref().clone());
    match repo.list_by_owner(tenant_id).await {
        Ok(shares) => web::HttpResponse::Ok().json(&ShareListResponse {
            success: true,
            data: shares,
        }),
        Err(e) => web::HttpResponse::InternalServerError().json(&serde_json::json!({
            "success": false,
            "message": e.to_string()
        })),
    }
}

/// GET /admin/shares/received - List shares received by current tenant
pub async fn list_received_shares(
    db: web::types::State<Arc<Database>>,
    claims: Claims,
) -> impl web::Responder {
    let tenant_id = claims.tenant_id;

    let repo = ShareRepository::new(db.get_ref().clone());
    match repo.list_received(tenant_id).await {
        Ok(shares) => web::HttpResponse::Ok().json(&ShareListResponse {
            success: true,
            data: shares,
        }),
        Err(e) => web::HttpResponse::InternalServerError().json(&serde_json::json!({
            "success": false,
            "message": e.to_string()
        })),
    }
}

/// POST /admin/shares - Create a new share
pub async fn create_share(
    db: web::types::State<Arc<Database>>,
    claims: Claims,
    body: web::types::Json<CreateShareRequest>,
) -> impl web::Responder {
    let tenant_id = claims.tenant_id;

    let cmd = CreateShareCmd {
        resource_type: body.resource_type.clone(),
        resource_id: body.resource_id,
        shared_to_tenant_id: body.shared_to_tenant_id,
        can_reshare: body.can_reshare,
    };

    let repo = ShareRepository::new(db.get_ref().clone());
    let user_id = Uuid::parse_str(&claims.sub).ok();
    
    match repo.create(cmd, tenant_id, user_id).await {
        Ok(share) => web::HttpResponse::Created().json(&ShareResponse {
            success: true,
            data: Some(share),
            message: None,
        }),
        Err(e) => web::HttpResponse::BadRequest().json(&serde_json::json!({
            "success": false,
            "message": e.to_string()
        })),
    }
}

/// DELETE /admin/shares/{id} - Revoke a share (checks usage first)
pub async fn revoke_share(
    db: web::types::State<Arc<Database>>,
    claims: Claims,
    path: web::types::Path<Uuid>,
    query: web::types::Query<ForceRevokeQuery>,
) -> impl web::Responder {
    let tenant_id = claims.tenant_id;
    let share_id = path.into_inner();
    let repo = ShareRepository::new(db.get_ref().clone());

    // Check if force revoke
    if query.force.unwrap_or(false) {
        match repo.force_revoke(share_id, tenant_id).await {
            Ok(revoked) => {
                if revoked {
                    web::HttpResponse::Ok().json(&RevokeResponse {
                        success: true,
                        message: "Share force revoked successfully".to_string(),
                        usage_count: None,
                    })
                } else {
                    web::HttpResponse::NotFound().json(&RevokeResponse {
                        success: false,
                        message: "Share not found or you don't have permission".to_string(),
                        usage_count: None,
                    })
                }
            }
            Err(e) => web::HttpResponse::InternalServerError().json(&serde_json::json!({
                "success": false,
                "message": e.to_string()
            })),
        }
    } else {
        // Normal revoke with usage check
        match repo.revoke(share_id, tenant_id).await {
            Ok((revoked, usage_count)) => {
                if revoked {
                    web::HttpResponse::Ok().json(&RevokeResponse {
                        success: true,
                        message: "Share revoked successfully".to_string(),
                        usage_count: None,
                    })
                } else {
                    web::HttpResponse::Conflict().json(&RevokeResponse {
                        success: false,
                        message: format!("Resource is still in use by {} entities. Use ?force=true to revoke anyway.", usage_count.unwrap_or(0)),
                        usage_count,
                    })
                }
            }
            Err(e) => web::HttpResponse::BadRequest().json(&serde_json::json!({
                "success": false,
                "message": e.to_string()
            })),
        }
    }
}

/// GET /admin/shares/{id}/usage - Get usage count for a share
pub async fn get_share_usage(
    db: web::types::State<Arc<Database>>,
    path: web::types::Path<Uuid>,
) -> impl web::Responder {
    let share_id = path.into_inner();
    let repo = ShareRepository::new(db.get_ref().clone());

    match repo.get_usage_count(share_id).await {
        Ok(count) => web::HttpResponse::Ok().json(&serde_json::json!({
            "success": true,
            "usage_count": count
        })),
        Err(e) => web::HttpResponse::NotFound().json(&serde_json::json!({
            "success": false,
            "message": e.to_string()
        })),
    }
}

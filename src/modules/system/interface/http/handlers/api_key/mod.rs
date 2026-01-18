use ntex::web;
use std::sync::Arc;
use crate::modules::system::application::services::api_key::ApiKeyService;
#[allow(unused_imports)]
use crate::modules::system::domain::api_key::ApiKey;
use crate::core::infrastructure::audit::AuditService;
use crate::core::utils::response::ApiResponse;
use crate::modules::system::interface::http::dto::api_key::CreateApiKeyRequest;
use serde_json::json;
use uuid::Uuid;

/// Create a new API key (Admin)
#[utoipa::path(
    post,
    path = "/api/v1/admin/api-keys",
    request_body = CreateApiKeyRequest,
    responses(
        (status = 201, description = "API key created successfully")
    ),
    tag = "api-keys",
    security(
        ("bearer_auth" = [])
    )
)]
pub async fn create_api_key(
    body: web::types::Json<CreateApiKeyRequest>,
    service: web::types::State<Arc<ApiKeyService>>,
    audit: web::types::State<Arc<AuditService>>,
) -> impl web::Responder {
    match service.create_key(body.tenant_id, body.name.clone(), &body.key_type, None).await {
        Ok((key, plain)) => {
            let _ = audit.log("SuperAdmin", "API_KEY_CREATED", Some(&key.id.to_string()), "SUCCESS", None).await;
            let response = ApiResponse::created(json!({
                "id": key.id,
                "prefix": key.prefix,
                "plain_key": plain,
                "note": "Save this key, it won't be shown again!"
            }), "API key created successfully");
            web::HttpResponse::Created().json(&response)
        }
        Err(e) => {
            let response = ApiResponse::<()>::internal_error(&e.message);
            web::HttpResponse::InternalServerError().json(&response)
        }
    }
}

/// List API keys (Hierarchical)
#[utoipa::path(
    get,
    path = "/api/v1/admin/api-keys",
    responses(
        (status = 200, description = "API keys retrieved successfully", body = Vec<ApiKey>)
    ),
    tag = "api-keys",
    security(
        ("bearer_auth" = [])
    )
)]
pub async fn list_api_keys(
    service: web::types::State<Arc<ApiKeyService>>,
    claims: crate::core::utils::jwt::Claims,
) -> impl web::Responder {
    let tenant_id = claims.tenant_id;
    match service.list_keys_hierarchical(tenant_id).await {
        Ok(keys) => {
            let response = ApiResponse::ok(json!({ "keys": keys }), "API keys listed successfully");
            web::HttpResponse::Ok().json(&response)
        }
        Err(e) => {
            let response = ApiResponse::<()>::internal_error(&e.message);
            web::HttpResponse::InternalServerError().json(&response)
        }
    }
}

/// Revoke an API key (Hierarchical)
#[utoipa::path(
    delete,
    path = "/api/v1/admin/api-keys/{id}",
    params(
        ("id" = Uuid, Path, description = "API Key ID to revoke")
    ),
    responses(
        (status = 200, description = "API key revoked successfully"),
        (status = 400, description = "Failed to revoke key or access denied")
    ),
    tag = "api-keys",
    security(
        ("bearer_auth" = [])
    )
)]
pub async fn revoke_api_key(
    path: web::types::Path<Uuid>,
    service: web::types::State<Arc<ApiKeyService>>,
    claims: crate::core::utils::jwt::Claims,
) -> impl web::Responder {
    let key_id = path.into_inner();
    let actor_tenant_id = claims.tenant_id;
    match service.revoke_key(key_id, actor_tenant_id).await {
        Ok(_) => {
            let response = ApiResponse::ok(json!({ "revoked": true }), "API key revoked successfully");
            web::HttpResponse::Ok().json(&response)
        }
        Err(e) => {
            let response = ApiResponse::<()>::bad_request(&e.message);
            web::HttpResponse::BadRequest().json(&response)
        }
    }
}

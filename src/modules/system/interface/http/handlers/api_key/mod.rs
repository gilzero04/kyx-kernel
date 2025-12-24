use ntex::web;
use std::sync::Arc;
use crate::modules::system::application::services::api_key::ApiKeyService;
use crate::core::infrastructure::audit::AuditService;
use crate::modules::system::interface::http::dto::api_key::CreateApiKeyRequest;

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

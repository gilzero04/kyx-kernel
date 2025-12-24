use ntex::web;
use std::sync::Arc;
use crate::modules::system::application::services::cors::CORSService;
use crate::core::infrastructure::audit::AuditService;
use crate::modules::system::interface::http::dto::cors::AddCorsRequest;

/// List allowed CORS origins (Admin)
#[utoipa::path(
    get,
    path = "/api/v1/admin/cors",
    responses(
        (status = 200, description = "List of CORS origins", body = Vec<CorsOrigin>)
    ),
    tag = "cors",
    security(
        ("bearer_auth" = [])
    )
)]
pub async fn list_cors_origins(service: web::types::State<Arc<CORSService>>) -> impl web::Responder {
    match service.list_origins().await {
        Ok(origins) => web::HttpResponse::Ok().json(&origins),
        Err(e) => web::HttpResponse::InternalServerError().json(&serde_json::json!({ "error": e.message }))
    }
}

/// Add a new allowed CORS origin (Admin)
#[utoipa::path(
    post,
    path = "/api/v1/admin/cors",
    request_body = AddCorsRequest,
    responses(
        (status = 201, description = "CORS origin added successfully", body = CorsOrigin)
    ),
    tag = "cors",
    security(
        ("bearer_auth" = [])
    )
)]
pub async fn add_cors_origin(
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

/// Delete an allowed CORS origin (Admin)
#[utoipa::path(
    delete,
    path = "/api/v1/admin/cors/{id}",
    responses(
        (status = 200, description = "CORS origin deleted successfully"),
        (status = 404, description = "CORS origin not found")
    ),
    tag = "cors",
    params(
        ("id" = i32, Path, description = "CORS origin ID")
    ),
    security(
        ("bearer_auth" = [])
    )
)]
pub async fn delete_cors_origin(
    service: web::types::State<Arc<CORSService>>,
    audit: web::types::State<Arc<AuditService>>,
    path: web::types::Path<i32>,
) -> impl web::Responder {
    let id = path.into_inner();
    match service.delete_origin(id).await {
        Ok(_) => {
            let _ = audit.log("SuperAdmin", "CORS_ORIGIN_DELETED", Some(&id.to_string()), "SUCCESS", None).await;
            web::HttpResponse::Ok().json(&serde_json::json!({ "success": true }))
        },
        Err(e) => {
            let mut status = if e.code == 404 { web::HttpResponse::NotFound() } else { web::HttpResponse::InternalServerError() };
            status.json(&serde_json::json!({ "error": e.message }))
        }
    }
}

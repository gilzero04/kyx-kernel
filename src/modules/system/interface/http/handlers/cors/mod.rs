use crate::core::infrastructure::audit::AuditService;
use crate::core::utils::response::ApiResponse;
use crate::modules::system::application::services::cors::CORSService;
use crate::modules::system::interface::http::dto::cors::{AddCorsRequest, UpdateCorsRequest};
use ntex::web;
use std::sync::Arc;

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
pub async fn list_cors_origins(
    service: web::types::State<Arc<CORSService>>,
) -> impl web::Responder {
    match service.list_origins().await {
        Ok(origins) => {
            let response = ApiResponse::ok(
                serde_json::json!({ "origins": origins }),
                "CORS origins retrieved",
            );
            web::HttpResponse::Ok().json(&response)
        }
        Err(e) => {
            let response = ApiResponse::<()>::internal_error(&e.message);
            web::HttpResponse::InternalServerError().json(&response)
        }
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
    match service
        .add_origin(&body.origin, body.description.clone())
        .await
    {
        Ok(origin) => {
            let _ = audit
                .log(
                    "SuperAdmin",
                    "CORS_ORIGIN_ADDED",
                    Some(&origin.origin),
                    "SUCCESS",
                    None,
                )
                .await;
            let response = ApiResponse::created(origin, "CORS origin added");
            web::HttpResponse::Created().json(&response)
        }
        Err(e) => {
            let response = ApiResponse::<()>::bad_request(&e.message);
            web::HttpResponse::BadRequest().json(&response)
        }
    }
}

/// Update a CORS origin (Admin)
#[utoipa::path(
    patch,
    path = "/api/v1/admin/cors/{id}",
    request_body = UpdateCorsRequest,
    responses(
        (status = 200, description = "CORS origin updated successfully", body = CorsOrigin),
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
pub async fn update_cors_origin(
    body: web::types::Json<UpdateCorsRequest>,
    service: web::types::State<Arc<CORSService>>,
    audit: web::types::State<Arc<AuditService>>,
    path: web::types::Path<i32>,
) -> impl web::Responder {
    let id = path.into_inner();
    match service
        .update_origin(id, body.is_active, body.description.clone())
        .await
    {
        Ok(origin) => {
            let _ = audit
                .log(
                    "SuperAdmin",
                    "CORS_ORIGIN_UPDATED",
                    Some(&origin.origin),
                    "SUCCESS",
                    None,
                )
                .await;
            let response = ApiResponse::ok(origin, "CORS origin updated");
            web::HttpResponse::Ok().json(&response)
        }
        Err(e) => {
            let response = if e.message.contains("no rows returned") {
                ApiResponse::<()>::not_found(&e.message)
            } else {
                ApiResponse::<()>::internal_error(&e.message)
            };
            let mut status = if e.message.contains("no rows returned") {
                web::HttpResponse::NotFound()
            } else {
                web::HttpResponse::InternalServerError()
            };
            status.json(&response)
        }
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
            let _ = audit
                .log(
                    "SuperAdmin",
                    "CORS_ORIGIN_DELETED",
                    Some(&id.to_string()),
                    "SUCCESS",
                    None,
                )
                .await;
            let response = ApiResponse::ok(
                serde_json::json!({ "deleted": true }),
                "CORS origin deleted",
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

use ntex::web;
use std::sync::Arc;
use crate::modules::system::application::services::tenant::TenantService;
use crate::modules::system::interface::http::dto::tenant::{TenantsQuery, UpdateOwnerRequest};
use crate::modules::system::domain::tenant::TenantFilter;
use crate::core::infrastructure::audit::AuditService;

/// List all tenants (SuperAdmin)
#[utoipa::path(
    get,
    path = "/api/v1/admin/tenants",
    params(
        TenantsQuery
    ),
    responses(
        (status = 200, description = "List of tenants", body = PaginatedTenants)
    ),
    tag = "tenants",
    security(
        ("bearer_auth" = [])
    )
)]
pub async fn list_tenants(
    service: web::types::State<Arc<TenantService>>,
    query: web::types::Query<TenantsQuery>,
) -> impl web::Responder {
    let filter = TenantFilter {
        page: query.page.unwrap_or(1),
        limit: query.limit.unwrap_or(10),
        sort: query.sort.clone(),
        search: query.search.clone(),
    };

    match service.list_tenants(filter).await {
        Ok(data) => web::HttpResponse::Ok().json(&data),
        Err(e) => web::HttpResponse::InternalServerError().json(&serde_json::json!({
            "error": e.message
        }))
    }
}

/// Update system owner name (Admin)
#[utoipa::path(
    patch,
    path = "/api/v1/admin/settings/owner",
    request_body = UpdateOwnerRequest,
    responses(
        (status = 200, description = "Owner updated successfully")
    ),
    tag = "tenants",
    security(
        ("bearer_auth" = [])
    )
)]
pub async fn update_owner(
    service: web::types::State<Arc<TenantService>>,
    audit: web::types::State<Arc<AuditService>>,
    body: web::types::Json<UpdateOwnerRequest>,
) -> impl web::Responder {
    if let Some(name) = &body.name {
        match service.update_owner(name.clone()).await {
            Ok(_) => {
                let _ = audit.log("SuperAdmin", "OWNER_UPDATED", Some("name"), "SUCCESS", None).await;
                web::HttpResponse::Ok().json(&serde_json::json!({ "success": true }))
            },
            Err(e) => web::HttpResponse::InternalServerError().json(&serde_json::json!({
                "error": e.message
            }))
        }
    } else {
        web::HttpResponse::BadRequest().json(&serde_json::json!({ "error": "No fields to update" }))
    }
}

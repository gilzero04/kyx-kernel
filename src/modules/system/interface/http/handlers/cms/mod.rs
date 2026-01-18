use crate::core::utils::jwt::Claims;
use crate::core::utils::response::ApiResponse;
use crate::modules::system::application::services::cms::CmsService;
use crate::modules::system::domain::cms::entity::PageEntry;
use chrono::Utc;
use ntex::web;
use serde::Deserialize;
use serde_json::json;
use std::sync::Arc;
use uuid::Uuid;

#[derive(Debug, Deserialize)]
pub struct CmsPageRequest {
    pub slug: String,
    pub title: String,
    pub content: serde_json::Value,
    pub is_published: bool,
}

// DELETED: Manual extraction replaced by direct Claims injection

/// Get page by slug for a tenant (Public)
#[utoipa::path(
    get,
    path = "/api/v1/public/cms/pages/{tenant_id}/{slug}",
    responses(
        (status = 200, description = "Page fetched successfully"),
        (status = 404, description = "Page not found")
    ),
    tag = "cms"
)]
pub async fn get_page_by_slug(
    req: web::HttpRequest,
    service: web::types::State<Arc<CmsService>>,
) -> Result<web::HttpResponse, web::Error> {
    // Manually parse path: .../cms/pages/{tenant_id}/{slug}
    let path_str = req.path();
    let segments: Vec<&str> = path_str.split("/cms/pages/").collect();

    if segments.len() < 2 {
        let response = ApiResponse::<()>::bad_request("Invalid CMS path");
        return Ok(web::HttpResponse::BadRequest().json(&response));
    }

    let remainder = segments[1];
    let parts: Vec<&str> = remainder.splitn(2, '/').collect();

    if parts.len() < 2 {
        let response = ApiResponse::<()>::not_found("Tenant ID and slug required");
        return Ok(web::HttpResponse::NotFound().json(&response));
    }

    let tenant_id = match Uuid::parse_str(parts[0]) {
        Ok(id) => id,
        Err(_) => {
            let response = ApiResponse::<()>::bad_request("Invalid tenant ID format");
            return Ok(web::HttpResponse::BadRequest().json(&response));
        }
    };

    let slug = parts[1].to_string();

    if slug.is_empty() {
        let response = ApiResponse::<()>::not_found("Page slug required");
        return Ok(web::HttpResponse::NotFound().json(&response));
    }

    // Strict tenant scoping: Only find pages belonging to the specified tenant
    match service.get_page_by_slug(tenant_id, &slug).await {
        Ok(Some(page)) => {
            let response = ApiResponse::ok(json!({ "page": page }), "Page fetched successfully");
            Ok(web::HttpResponse::Ok().json(&response))
        }
        Ok(None) => {
            let response = ApiResponse::<()>::not_found("Page not found");
            Ok(web::HttpResponse::NotFound().json(&response))
        }
        Err(e) => {
            let response = ApiResponse::<()>::internal_error(&e.message);
            Ok(web::HttpResponse::InternalServerError().json(&response))
        }
    }
}

/// List all pages for the current tenant (Admin)
pub async fn list_admin_pages(
    service: web::types::State<Arc<CmsService>>,
    claims: Claims,
) -> Result<web::HttpResponse, web::Error> {
    let tenant_id = claims.tenant_id;

    match service.list_pages(tenant_id).await {
        Ok(pages) => {
            let response = ApiResponse::ok(json!({ "pages": pages }), "Pages listed successfully");
            Ok(web::HttpResponse::Ok().json(&response))
        }
        Err(e) => {
            let response = ApiResponse::<()>::internal_error(&e.message);
            Ok(web::HttpResponse::InternalServerError().json(&response))
        }
    }
}

/// Get a single CMS page detail (Admin)
pub async fn get_admin_page(
    path: web::types::Path<Uuid>,
    service: web::types::State<Arc<CmsService>>,
    claims: Claims,
) -> Result<web::HttpResponse, web::Error> {
    let page_id = path.into_inner();
    let tenant_id = claims.tenant_id;

    match service.get_page_by_id(tenant_id, page_id).await {
        Ok(Some(page)) => {
            let response = ApiResponse::ok(json!({ "page": page }), "Page fetched successfully");
            Ok(web::HttpResponse::Ok().json(&response))
        }
        Ok(None) => {
            let response = ApiResponse::<()>::not_found("Page not found");
            Ok(web::HttpResponse::NotFound().json(&response))
        }
        Err(e) => {
            let response = ApiResponse::<()>::internal_error(&e.message);
            Ok(web::HttpResponse::InternalServerError().json(&response))
        }
    }
}

/// Create a new CMS page
pub async fn create_admin_page(
    body: web::types::Json<CmsPageRequest>,
    service: web::types::State<Arc<CmsService>>,
    claims: Claims,
) -> Result<web::HttpResponse, web::Error> {
    let tenant_id = claims.tenant_id;
    let new_id = Uuid::new_v4();

    let page = PageEntry {
        id: new_id,
        tenant_id,
        slug: body.slug.clone(),
        title: body.title.clone(),
        content: body.content.clone(),
        is_published: body.is_published,
        created_at: Utc::now(),
        updated_at: Utc::now(),
    };

    match service.save_page(page).await {
        Ok(_) => {
            let response =
                ApiResponse::created(json!({ "id": new_id }), "Page created successfully");
            Ok(web::HttpResponse::Created().json(&response))
        }
        Err(e) => {
            let response = ApiResponse::<()>::internal_error(&e.message);
            Ok(web::HttpResponse::InternalServerError().json(&response))
        }
    }
}

/// Update an existing CMS page
pub async fn update_admin_page(
    path: web::types::Path<Uuid>,
    body: web::types::Json<CmsPageRequest>,
    service: web::types::State<Arc<CmsService>>,
    claims: Claims,
) -> Result<web::HttpResponse, web::Error> {
    let page_id = path.into_inner();
    let tenant_id = claims.tenant_id;

    // Verify ownership first
    let existing = match service.get_page_by_id(tenant_id, page_id).await {
        Ok(Some(p)) => p,
        Ok(None) => {
            let response = ApiResponse::<()>::not_found("Page not found");
            return Ok(web::HttpResponse::NotFound().json(&response));
        }
        Err(e) => {
            let response = ApiResponse::<()>::internal_error(&e.message);
            return Ok(web::HttpResponse::InternalServerError().json(&response));
        }
    };

    let updated_page = PageEntry {
        id: page_id,
        tenant_id,
        slug: body.slug.clone(),
        title: body.title.clone(),
        content: body.content.clone(),
        is_published: body.is_published,
        created_at: existing.created_at,
        updated_at: Utc::now(),
    };

    match service.save_page(updated_page).await {
        Ok(_) => {
            let response = ApiResponse::ok(
                json!({ "updated": true, "id": page_id }),
                "Page updated successfully",
            );
            Ok(web::HttpResponse::Ok().json(&response))
        }
        Err(e) => {
            let response = ApiResponse::<()>::internal_error(&e.message);
            Ok(web::HttpResponse::InternalServerError().json(&response))
        }
    }
}

/// Delete a CMS page
pub async fn delete_admin_page(
    path: web::types::Path<Uuid>,
    service: web::types::State<Arc<CmsService>>,
    claims: Claims,
) -> Result<web::HttpResponse, web::Error> {
    let page_id = path.into_inner();
    let tenant_id = claims.tenant_id;

    match service.delete_page(tenant_id, page_id).await {
        Ok(_) => {
            let response = ApiResponse::ok(json!({ "deleted": true }), "Page deleted successfully");
            Ok(web::HttpResponse::Ok().json(&response))
        }
        Err(e) => {
            let response = ApiResponse::<()>::internal_error(&e.message);
            Ok(web::HttpResponse::InternalServerError().json(&response))
        }
    }
}

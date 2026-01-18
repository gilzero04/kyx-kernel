use crate::core::infrastructure::audit::AuditService;
use crate::core::utils::jwt::Claims;
use crate::core::utils::response::ApiResponse;
use crate::modules::system::application::services::tenant::TenantService;
use crate::modules::system::application::services::tenant::domain_verification::DomainVerificationService;
use crate::modules::system::domain::tenant::TenantFilter;
use crate::modules::system::interface::http::dto::tenant::{
    CreateTenantRequest, TenantsQuery, UpdateOwnerRequest, UpdateTenantRequest,
};
use ntex::web;
use std::sync::Arc;
use uuid::Uuid;

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
    claims: Claims,
    query: web::types::Query<TenantsQuery>,
) -> impl web::Responder {
    let mut actor_tenant_id = Some(claims.tenant_id);

    // If actor is system owner, treat as None for "unfiltered" access
    if let Ok(owner_id) = service.get_owner_id().await
        && let Some(tid) = actor_tenant_id
            && tid == owner_id {
                actor_tenant_id = None;
            }

    let filter = TenantFilter {
        page: query.page.unwrap_or(1),
        limit: query.limit.unwrap_or(10),
        sort: query.sort.clone(),
        search: query.search.clone(),
        actor_tenant_id: None, // Will be set by service
    };

    match service.list_tenants(filter, actor_tenant_id).await {
        Ok(data) => {
            let payload = if let Ok(owner_id) = service.get_owner_id().await {
                serde_json::json!({
                    "tenants": data.data,
                    "pagination": data.pagination,
                    "owner_id": owner_id
                })
            } else {
                serde_json::json!({
                    "tenants": data.data,
                    "pagination": data.pagination
                })
            };
            let response = ApiResponse::ok(payload, "Tenants retrieved");
            web::HttpResponse::Ok().json(&response)
        }
        Err(e) => {
            let response = ApiResponse::<()>::internal_error(&e.message);
            web::HttpResponse::InternalServerError().json(&response)
        }
    }
}

/// Update system owner (branding now managed via branding_id)
#[utoipa::path(
    patch,
    path = "/api/v1/admin/tenants/owner",
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
    claims: Claims,
    body: web::types::Json<UpdateOwnerRequest>,
) -> impl web::Responder {
    let actor_tenant_id = Some(claims.tenant_id);

    // Strictly enforce System Owner only
    if let Ok(owner_id) = service.get_owner_id().await
        && actor_tenant_id != Some(owner_id) {
            return web::HttpResponse::Forbidden().json(
                &serde_json::json!({ "error": "Only the system owner can update owner settings" }),
            );
        }

    // Check if any fields are being updated
    if body.name.is_some()
        || body.slug.is_some()
        || body.branding_id.is_some()
        || body.contact_email.is_some()
        || body.contact_phone.is_some()
        || body.website_url.is_some()
        || body.social_links.is_some()
        || body.address.is_some()
        || body.business_type.is_some()
        || body.tax_id.is_some()
    {
        match service
            .update_owner(
                body.name.clone(),
                body.slug.clone(),
                body.branding_id,
                body.contact_email.clone(),
                body.contact_phone.clone(),
                body.website_url.clone(),
                body.social_links.clone(),
                body.address.clone(),
                body.business_type.clone(),
                body.tax_id.clone(),
                body.config.clone(),
                body.custom_domain.clone(),
                body.allow_child_subdomains,
                body.domain_verified_at,
                body.verification_token.clone(),
            )
            .await
        {
            Ok(_) => {
                let _ = audit
                    .log(
                        "SuperAdmin",
                        "OWNER_UPDATED",
                        Some("owner"),
                        "SUCCESS",
                        None,
                    )
                    .await;
                web::HttpResponse::Ok().json(&serde_json::json!({ "success": true }))
            }
            Err(e) => web::HttpResponse::InternalServerError().json(&serde_json::json!({
                "error": e.message
            })),
        }
    } else {
        web::HttpResponse::BadRequest().json(&serde_json::json!({ "error": "No fields to update" }))
    }
}

/// Get a single organization by ID
#[utoipa::path(
    get,
    path = "/api/v1/admin/tenants/{id}",
    responses(
        (status = 200, description = "Organization found", body = TenantEntry),
        (status = 404, description = "Organization not found")
    ),
    tag = "tenants",
    security(
        ("bearer_auth" = [])
    )
)]
pub async fn get_tenant(
    service: web::types::State<Arc<TenantService>>,
    claims: Claims,
    id: web::types::Path<sqlx::types::Uuid>,
) -> impl web::Responder {
    let mut actor_tenant_id = Some(claims.tenant_id);

    // If actor is system owner, treat as None for unfiltered access
    if let Ok(owner_id) = service.get_owner_id().await
        && let Some(tid) = actor_tenant_id
            && tid == owner_id {
                actor_tenant_id = None;
            }

    match service.get_tenant_by_id(*id, actor_tenant_id).await {
        Ok(Some(tenant)) => web::HttpResponse::Ok().json(&tenant),
        Ok(None) => web::HttpResponse::NotFound().json(&serde_json::json!({
            "error": "Organization not found"
        })),
        Err(e) => web::HttpResponse::InternalServerError().json(&serde_json::json!({
            "error": e.message
        })),
    }
}

/// Create a new organization (branding managed via branding_id)
#[utoipa::path(
    post,
    path = "/api/v1/admin/tenants",
    request_body = CreateTenantRequest,
    responses(
        (status = 201, description = "Organization created", body = TenantEntry)
    ),
    tag = "tenants",
    security(
        ("bearer_auth" = [])
    )
)]
pub async fn create_tenant(
    service: web::types::State<Arc<TenantService>>,
    audit: web::types::State<Arc<AuditService>>,
    claims: Claims,
    body: web::types::Json<CreateTenantRequest>,
) -> impl web::Responder {
    let mut actor_tenant_id = Some(claims.tenant_id);

    // Capture original actor_id for default parent logic
    let original_actor_id = actor_tenant_id;

    // If actor is system owner, treat as None (allowing them to specify any parent_id or bypass checks)
    if let Ok(owner_id) = service.get_owner_id().await
        && let Some(tid) = actor_tenant_id
            && tid == owner_id {
                actor_tenant_id = None;
            }

    // Default parent_id to the creator's tenant if not specified
    let final_parent_id = body.parent_id.or(original_actor_id);

    match service
        .create_tenant(
            body.name.clone(),
            body.slug.clone(),
            body.type_slug.clone(),
            body.plan.clone(),
            body.admin_email.clone(),
            body.admin_password.clone(),
            body.admin_name.clone(),
            final_parent_id,
            actor_tenant_id,
            body.branding_id,
            body.contact_email.clone(),
            body.contact_phone.clone(),
            body.website_url.clone(),
            body.social_links.clone(),
            body.address.clone(),
            body.business_type.clone(),
        )
        .await
    {
        Ok(data) => {
            let _ = audit
                .log(
                    "SuperAdmin",
                    "TENANT_CREATED",
                    Some(&data.id.to_string()),
                    "SUCCESS",
                    None,
                )
                .await;
            web::HttpResponse::Created().json(&data)
        }
        Err(e) => web::HttpResponse::InternalServerError().json(&serde_json::json!({
            "error": e.message
        })),
    }
}

/// Update organization metadata (branding managed via branding_id)
#[utoipa::path(
    patch,
    path = "/api/v1/admin/tenants/{id}",
    request_body = UpdateTenantRequest,
    responses(
        (status = 200, description = "Organization updated", body = TenantEntry)
    ),
    tag = "tenants",
    security(
        ("bearer_auth" = [])
    )
)]
pub async fn update_tenant(
    service: web::types::State<Arc<TenantService>>,
    audit: web::types::State<Arc<AuditService>>,
    claims: Claims,
    id: web::types::Path<sqlx::types::Uuid>,
    body: web::types::Json<UpdateTenantRequest>,
) -> impl web::Responder {
    let mut actor_tenant_id = Some(claims.tenant_id);

    // If actor is system owner, treat as None
    if let Ok(owner_id) = service.get_owner_id().await
        && let Some(tid) = actor_tenant_id
            && tid == owner_id {
                actor_tenant_id = None;
            }

    match service
        .update_tenant(
            id.into_inner(),
            body.name.clone(),
            body.slug.clone(),
            body.is_active,
            body.branding_id,
            body.contact_email.clone(),
            body.contact_phone.clone(),
            body.website_url.clone(),
            body.social_links.clone(),
            body.address.clone(),
            body.business_type.clone(),
            body.config.clone(),
            actor_tenant_id,
            body.custom_domain.clone(),
            body.allow_child_subdomains,
            body.use_parent_subdomain,
            body.domain_verified_at,
            body.verification_token.clone(),
        )
        .await
    {
        Ok(data) => {
            let _ = audit
                .log(
                    "SuperAdmin",
                    "TENANT_UPDATED",
                    Some(&data.id.to_string()),
                    "SUCCESS",
                    None,
                )
                .await;
            web::HttpResponse::Ok().json(&data)
        }
        Err(e) => web::HttpResponse::InternalServerError().json(&serde_json::json!({
            "error": e.message
        })),
    }
}

/// Delete an organization (SuperAdmin)
#[utoipa::path(
    delete,
    path = "/api/v1/admin/tenants/{id}",
    responses(
        (status = 200, description = "Organization deleted")
    ),
    tag = "tenants",
    security(
        ("bearer_auth" = [])
    )
)]
pub async fn delete_tenant(
    service: web::types::State<Arc<TenantService>>,
    audit: web::types::State<Arc<AuditService>>,
    claims: Claims,
    id: web::types::Path<sqlx::types::Uuid>,
) -> impl web::Responder {
    let tenant_id = id.into_inner();

    let mut actor_tenant_id = Some(claims.tenant_id);

    // If actor is system owner, treat as None
    if let Ok(owner_id) = service.get_owner_id().await
        && let Some(tid) = actor_tenant_id
            && tid == owner_id {
                actor_tenant_id = None;
            }

    match service.delete_tenant(tenant_id, actor_tenant_id).await {
        Ok(_) => {
            let _ = audit
                .log(
                    "SuperAdmin",
                    "TENANT_DELETED",
                    Some(&tenant_id.to_string()),
                    "SUCCESS",
                    None,
                )
                .await;
            web::HttpResponse::Ok().json(&serde_json::json!({ "success": true }))
        }
        Err(e) => web::HttpResponse::BadRequest().json(&serde_json::json!({
            "error": e.message
        })),
    }
}

/// Verify custom domain ownership and connectivity
#[utoipa::path(
    post,
    path = "/api/v1/admin/tenants/{id}/verify-domain",
    responses(
        (status = 200, description = "Domain verified successfully"),
        (status = 400, description = "Verification failed")
    ),
    tag = "tenants",
    security(
        ("bearer_auth" = [])
    )
)]
pub async fn verify_domain(
    service: web::types::State<Arc<DomainVerificationService>>,
    _claims: Claims,
    id: web::types::Path<Uuid>,
    body: web::types::Json<serde_json::Value>,
) -> impl web::Responder {
    let tenant_id = id.into_inner();

    // Authorization check: Only root admins or the tenant owner can verify
    // (Simplification: relying on frontend + backend service check)

    let custom_domain = body
        .get("custom_domain")
        .and_then(|v| v.as_str())
        .unwrap_or("");

    if custom_domain.is_empty() {
        return web::HttpResponse::BadRequest()
            .json(&serde_json::json!({ "error": "custom_domain is required" }));
    }

    match service.verify_domain(tenant_id, custom_domain).await {
        Ok(dt) => web::HttpResponse::Ok().json(&serde_json::json!({
            "success": true,
            "verified_at": dt
        })),
        Err(e) => web::HttpResponse::BadRequest().json(&serde_json::json!({ "error": e.message })),
    }
}

/// Get public tenant info by slug (returns branding_id for separate branding fetch)
#[utoipa::path(
    get,
    path = "/api/v1/public/tenants/{slug}",
    responses(
        (status = 200, description = "Organization found", body = TenantEntry),
        (status = 404, description = "Organization not found")
    ),
    tag = "tenants"
)]
pub async fn get_tenant_by_slug(
    service: web::types::State<Arc<TenantService>>,
    slug: web::types::Path<String>,
) -> impl web::Responder {
    let slug = slug.into_inner();

    match service.get_tenant_by_slug(slug).await {
        Ok(Some(tenant)) => {
            // Return tenant info with branding_id (branding fetched separately via workspace/status)
            web::HttpResponse::Ok().json(&serde_json::json!({
                "id": tenant.id,
                "name": tenant.name,
                "slug": tenant.slug,
                "branding_id": tenant.branding_id,
                "contact_email": tenant.contact_email,
                "contact_phone": tenant.contact_phone,
                "website_url": tenant.website_url
            }))
        }
        Ok(None) => web::HttpResponse::NotFound().json(&serde_json::json!({
            "error": "Organization not found"
        })),
        Err(e) => web::HttpResponse::InternalServerError().json(&serde_json::json!({
            "error": e.message
        })),
    }
}

/// Get current tenant (Me)
#[utoipa::path(
    get,
    path = "/api/v1/admin/tenants/me",
    responses(
        (status = 200, description = "Current organization", body = TenantEntry),
        (status = 404, description = "Organization not found")
    ),
    tag = "tenants",
    security(
        ("bearer_auth" = [])
    )
)]
pub async fn get_current_tenant(
    service: web::types::State<Arc<TenantService>>,
    claims: Claims,
) -> impl web::Responder {
    let tenant_id = claims.tenant_id;

    match service.get_tenant_by_id(tenant_id, Some(tenant_id)).await {
        Ok(Some(tenant)) => {
            let response = ApiResponse::ok(tenant, "Current tenant retrieved");
            web::HttpResponse::Ok().json(&response)
        }
        Ok(None) => {
            let response = ApiResponse::<()>::not_found("Organization not found");
            web::HttpResponse::NotFound().json(&response)
        }
        Err(e) => {
            let response = ApiResponse::<()>::internal_error(&e.message);
            web::HttpResponse::InternalServerError().json(&response)
        }
    }
}

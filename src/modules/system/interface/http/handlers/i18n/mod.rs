use crate::core::infrastructure::audit::AuditService;
use crate::core::utils::response::ApiResponse;
use crate::modules::system::application::services::i18n::I18nService;
use ntex::web;
use serde::Deserialize;
use serde_json::Value;
use std::sync::Arc;

use crate::modules::system::interface::http::dto::i18n::{
    CreateI18nKeyRequest, TranslationsResponse,
};

// === Query Parameters ===

#[derive(Debug, Deserialize)]
pub struct TranslationsQuery {
    pub tenant_id: Option<String>,
    pub context: Option<String>, // 'console' or 'workspace'
}

// === Handlers ===

/// List all available i18n locales
#[utoipa::path(
    get,
    path = "/api/v1/public/i18n/locales",
    responses(
        (status = 200, description = "List of locales")
    ),
    tag = "i18n"
)]
pub async fn list_locales(service: web::types::State<Arc<I18nService>>) -> impl web::Responder {
    match service.list_locales().await {
        Ok(data) => {
            let response =
                ApiResponse::ok(serde_json::json!({ "locales": data }), "Locales retrieved");
            web::HttpResponse::Ok().json(&response)
        }
        Err(e) => {
            let response = ApiResponse::<()>::internal_error(&e.to_string());
            web::HttpResponse::InternalServerError().json(&response)
        }
    }
}

/// Get translations for a specific locale
///
/// Query parameters:
/// - tenant_id: Optional tenant UUID for tenant-specific translations
/// - context: Optional context ('console' or 'workspace') for context-specific translations
#[utoipa::path(
    get,
    path = "/api/v1/public/i18n/translations/{locale}",
    responses(
        (status = 200, description = "Translations fetched successfully", body = TranslationsResponse)
    ),
    tag = "i18n",
    params(
        ("locale" = String, Path, description = "Locale code (e.g. en, th)"),
        ("tenant_id" = Option<String>, Query, description = "Tenant ID for tenant-specific translations"),
        ("context" = Option<String>, Query, description = "Context: 'console' or 'workspace'")
    )
)]
pub async fn get_translations(
    service: web::types::State<Arc<I18nService>>,
    path: web::types::Path<String>,
    query: web::types::Query<TranslationsQuery>,
) -> impl web::Responder {
    let locale = path.into_inner();

    // Parse tenant_id if provided
    let tenant_id = query
        .tenant_id
        .as_ref()
        .and_then(|id| uuid::Uuid::parse_str(id).ok())
        .map(|u| sqlx::types::Uuid::from_u128(u.as_u128()));

    let context = query.context.as_deref();

    match service
        .get_translations_map(&locale, tenant_id, context)
        .await
    {
        Ok(map) => {
            let data = TranslationsResponse {
                locale,
                translations: map,
            };
            let response = ApiResponse::ok(data, "Translations retrieved");
            web::HttpResponse::Ok().json(&response)
        }
        Err(e) => {
            let response = ApiResponse::<()>::internal_error(&e.to_string());
            web::HttpResponse::InternalServerError().json(&response)
        }
    }
}

/// Create a new i18n key (Admin)
#[utoipa::path(
    post,
    path = "/api/v1/admin/i18n/keys",
    request_body = CreateI18nKeyRequest,
    responses(
        (status = 200, description = "Key created successfully")
    ),
    tag = "i18n",
    security(
        ("bearer_auth" = [])
    )
)]
pub async fn create_key(
    service: web::types::State<Arc<I18nService>>,
    audit: web::types::State<Arc<AuditService>>,
    body: web::types::Json<CreateI18nKeyRequest>,
) -> impl web::Responder {
    // Parse optional tenant_id and context from body
    let tenant_id = body
        .tenant_id
        .as_ref()
        .and_then(|id| uuid::Uuid::parse_str(id).ok())
        .map(|u| sqlx::types::Uuid::from_u128(u.as_u128()));
    let context = body.context.as_deref();

    match service
        .create_key(&body.key, &body.default_message, tenant_id, context)
        .await
    {
        Ok(_) => {
            let _ = audit
                .log(
                    "System",
                    "I18N_KEY_CREATED",
                    Some(&body.key),
                    "SUCCESS",
                    None,
                )
                .await;
            let response = ApiResponse::ok(
                serde_json::json!({ "created": true, "key": &body.key }),
                "Key created",
            );
            web::HttpResponse::Ok().json(&response)
        }
        Err(e) => {
            let response = ApiResponse::<()>::internal_error(&e.to_string());
            web::HttpResponse::InternalServerError().json(&response)
        }
    }
}

/// Update an i18n translation (Admin)
#[utoipa::path(
    patch,
    path = "/api/v1/admin/i18n/translations",
    responses(
        (status = 200, description = "Translation updated successfully")
    ),
    tag = "i18n",
    security(
        ("bearer_auth" = [])
    )
)]
pub async fn update_translation(
    service: web::types::State<Arc<I18nService>>,
    audit: web::types::State<Arc<AuditService>>,
    body: web::types::Json<Value>, // Expect { "locale": "en", "key": "foo", "message": "bar", "tenant_id"?: "uuid", "context"?: "console" }
) -> impl web::Responder {
    let locale = body.get("locale").and_then(|v| v.as_str());
    let key = body.get("key").and_then(|v| v.as_str());
    let message = body.get("message").and_then(|v| v.as_str());
    let tenant_id_str = body.get("tenant_id").and_then(|v| v.as_str());
    let context = body.get("context").and_then(|v| v.as_str());

    // Parse tenant_id
    let tenant_id = tenant_id_str
        .and_then(|id| uuid::Uuid::parse_str(id).ok())
        .map(|u| sqlx::types::Uuid::from_u128(u.as_u128()));

    if let (Some(l), Some(k), Some(m)) = (locale, key, message) {
        match service
            .update_translation(l, k, m, tenant_id, context)
            .await
        {
            Ok(_) => {
                let _ = audit
                    .log(
                        "Admin",
                        "I18N_UPDATED",
                        Some(k),
                        "SUCCESS",
                        Some(serde_json::json!({ "locale": l, "message": m, "context": context })),
                    )
                    .await;
                let response = ApiResponse::ok(
                    serde_json::json!({ "updated": true, "key": k, "locale": l }),
                    "Translation updated",
                );
                web::HttpResponse::Ok().json(&response)
            }
            Err(e) => {
                let response = ApiResponse::<()>::internal_error(&e.to_string());
                web::HttpResponse::InternalServerError().json(&response)
            }
        }
    } else {
        let response = ApiResponse::<()>::bad_request("Missing locale, key, or message");
        web::HttpResponse::BadRequest().json(&response)
    }
}

/// Create a new locale (Admin)
#[utoipa::path(
    post,
    path = "/api/v1/admin/i18n/locales",
    request_body = CreateLocaleRequest,
    responses(
        (status = 200, description = "Locale created successfully")
    ),
    tag = "i18n",
    security(
        ("bearer_auth" = [])
    )
)]
pub async fn create_locale(
    service: web::types::State<Arc<I18nService>>,
    audit: web::types::State<Arc<AuditService>>,
    body: web::types::Json<crate::modules::system::interface::http::dto::i18n::CreateLocaleRequest>,
) -> impl web::Responder {
    match service.create_locale(&body.code, &body.name).await {
        Ok(_) => {
            let _ = audit
                .log(
                    "Admin",
                    "I18N_LOCALE_CREATED",
                    Some(&body.code),
                    "SUCCESS",
                    None,
                )
                .await;
            let response = ApiResponse::ok(
                serde_json::json!({ "created": true, "code": &body.code }),
                "Locale created",
            );
            web::HttpResponse::Ok().json(&response)
        }
        Err(e) => {
            let response = ApiResponse::<()>::internal_error(&e.to_string());
            web::HttpResponse::InternalServerError().json(&response)
        }
    }
}

/// Delete an i18n key and all its translations (Admin)
#[utoipa::path(
    delete,
    path = "/api/v1/admin/i18n/keys/{key}",
    responses(
        (status = 200, description = "Key deleted successfully")
    ),
    tag = "i18n",
    params(
        ("key" = String, Path, description = "Translation key to delete"),
        ("tenant_id" = Option<String>, Query, description = "Tenant ID to delete key for (omit for global)"),
        ("context" = Option<String>, Query, description = "Context to delete key for")
    ),
    security(
        ("bearer_auth" = [])
    )
)]
pub async fn delete_key(
    service: web::types::State<Arc<I18nService>>,
    audit: web::types::State<Arc<AuditService>>,
    path: web::types::Path<String>,
    query: web::types::Query<TranslationsQuery>,
) -> impl web::Responder {
    let key = path.into_inner();

    let tenant_id = query
        .tenant_id
        .as_ref()
        .and_then(|id| uuid::Uuid::parse_str(id).ok())
        .map(|u| sqlx::types::Uuid::from_u128(u.as_u128()));
    let context = query.context.as_deref();

    match service.delete_key(&key, tenant_id, context).await {
        Ok(_) => {
            let _ = audit
                .log("Admin", "I18N_KEY_DELETED", Some(&key), "SUCCESS", None)
                .await;
            let response = ApiResponse::ok(
                serde_json::json!({ "deleted": true, "key": &key }),
                "Key deleted",
            );
            web::HttpResponse::Ok().json(&response)
        }
        Err(e) => {
            let response = ApiResponse::<()>::internal_error(&e.to_string());
            web::HttpResponse::InternalServerError().json(&response)
        }
    }
}

/// Delete an i18n locale (Admin)
#[utoipa::path(
    delete,
    path = "/api/v1/admin/i18n/locales/{code}",
    responses(
        (status = 200, description = "Locale deleted successfully")
    ),
    tag = "i18n",
    params(
        ("code" = String, Path, description = "Locale code to delete (e.g. th)")
    ),
    security(
        ("bearer_auth" = [])
    )
)]
pub async fn delete_locale(
    _service: web::types::State<Arc<AuditService>>,
    audit: web::types::State<Arc<AuditService>>,
    i18n_service: web::types::State<Arc<I18nService>>,
    path: web::types::Path<String>,
) -> impl web::Responder {
    let code = path.into_inner();
    match i18n_service.delete_locale(&code).await {
        Ok(_) => {
            let _ = audit
                .log("Admin", "I18N_LOCALE_DELETED", Some(&code), "SUCCESS", None)
                .await;
            let response = ApiResponse::ok(
                serde_json::json!({ "deleted": true, "code": &code }),
                "Locale deleted",
            );
            web::HttpResponse::Ok().json(&response)
        }
        Err(e) => {
            let response = ApiResponse::<()>::internal_error(&e.to_string());
            web::HttpResponse::InternalServerError().json(&response)
        }
    }
}

/// List all translations for admin management
#[utoipa::path(
    get,
    path = "/api/v1/admin/i18n/translations",
    responses(
        (status = 200, description = "All translations retrieved")
    ),
    tag = "i18n",
    security(
        ("bearer_auth" = [])
    )
)]
pub async fn list_all_translations(
    service: web::types::State<Arc<I18nService>>,
) -> impl web::Responder {
    match service.list_all_translations().await {
        Ok(data) => {
            let response = ApiResponse::ok(
                serde_json::json!({ "translations": data }),
                "Translations retrieved",
            );
            web::HttpResponse::Ok().json(&response)
        }
        Err(e) => {
            let response = ApiResponse::<()>::internal_error(&e.to_string());
            web::HttpResponse::InternalServerError().json(&response)
        }
    }
}

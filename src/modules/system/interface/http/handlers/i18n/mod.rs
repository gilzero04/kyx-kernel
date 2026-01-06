use ntex::web;
use crate::modules::system::application::services::i18n::I18nService;
use crate::core::infrastructure::audit::AuditService;
use std::sync::Arc;
use serde_json::Value;

use crate::modules::system::interface::http::dto::i18n::{TranslationsResponse, CreateI18nKeyRequest};

// === Handlers ===

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
pub async fn list_locales(
    service: web::types::State<Arc<I18nService>>,
) -> impl web::Responder {
    match service.list_locales().await {
        Ok(data) => web::HttpResponse::Ok().json(&data),
        Err(e) => web::HttpResponse::InternalServerError().json(&serde_json::json!({ "error": e.to_string() })),
    }
}

/// Get translations for a specific locale
#[utoipa::path(
    get,
    path = "/api/v1/public/i18n/translations/{locale}",
    responses(
        (status = 200, description = "Translations fetched successfully", body = TranslationsResponse)
    ),
    tag = "i18n",
    params(
        ("locale" = String, Path, description = "Locale code (e.g. en, th)")
    )
)]
pub async fn get_translations(
    service: web::types::State<Arc<I18nService>>,
    path: web::types::Path<String>,
) -> impl web::Responder {
    let locale = path.into_inner();
    
    match service.get_translations_map(&locale).await {
        Ok(map) => web::HttpResponse::Ok().json(&TranslationsResponse {
            locale,
            translations: map,
        }),
        Err(e) => web::HttpResponse::InternalServerError().json(&serde_json::json!({ "error": e.to_string() })),
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
    match service.create_key(&body.key, &body.default_message).await {
        Ok(_) => {
            let _ = audit.log("System", "I18N_KEY_CREATED", Some(&body.key), "SUCCESS", None).await;
            web::HttpResponse::Ok().json(&serde_json::json!({ "success": true }))
        },
        Err(e) => web::HttpResponse::InternalServerError().json(&serde_json::json!({ "error": e.to_string() }))
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
    body: web::types::Json<Value>, // Expect { "locale": "en", "key": "foo", "message": "bar" }
) -> impl web::Responder {
    let locale = body.get("locale").and_then(|v| v.as_str());
    let key = body.get("key").and_then(|v| v.as_str());
    let message = body.get("message").and_then(|v| v.as_str());

    if let (Some(l), Some(k), Some(m)) = (locale, key, message) {
        match service.update_translation(l, k, m).await {
            Ok(_) => {
                let _ = audit.log("Admin", "I18N_UPDATED", Some(k), "SUCCESS", Some(serde_json::json!({ "locale": l, "message": m }))).await;
                web::HttpResponse::Ok().json(&serde_json::json!({ "success": true }))
            },
            Err(e) => web::HttpResponse::InternalServerError().json(&serde_json::json!({ "error": e.to_string() }))
        }
    } else {
        web::HttpResponse::BadRequest().json(&serde_json::json!({ "error": "Missing locale, key, or message" }))
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
            let _ = audit.log("Admin", "I18N_LOCALE_CREATED", Some(&body.code), "SUCCESS", None).await;
            web::HttpResponse::Ok().json(&serde_json::json!({ "success": true }))
        },
        Err(e) => web::HttpResponse::InternalServerError().json(&serde_json::json!({ "error": e.to_string() }))
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
        ("key" = String, Path, description = "Translation key to delete")
    ),
    security(
        ("bearer_auth" = [])
    )
)]
pub async fn delete_key(
    service: web::types::State<Arc<I18nService>>,
    audit: web::types::State<Arc<AuditService>>,
    path: web::types::Path<String>,
) -> impl web::Responder {
    let key = path.into_inner();
    match service.delete_key(&key).await {
        Ok(_) => {
            let _ = audit.log("Admin", "I18N_KEY_DELETED", Some(&key), "SUCCESS", None).await;
            web::HttpResponse::Ok().json(&serde_json::json!({ "success": true }))
        },
        Err(e) => web::HttpResponse::InternalServerError().json(&serde_json::json!({ "error": e.to_string() }))
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
    service: web::types::State<Arc<I18nService>>,
    audit: web::types::State<Arc<AuditService>>,
    path: web::types::Path<String>,
) -> impl web::Responder {
    let code = path.into_inner();
    match service.delete_locale(&code).await {
        Ok(_) => {
            let _ = audit.log("Admin", "I18N_LOCALE_DELETED", Some(&code), "SUCCESS", None).await;
            web::HttpResponse::Ok().json(&serde_json::json!({ "success": true }))
        },
        Err(e) => web::HttpResponse::InternalServerError().json(&serde_json::json!({ "error": e.to_string() }))
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
        Ok(data) => web::HttpResponse::Ok().json(&data),
        Err(e) => web::HttpResponse::InternalServerError().json(&serde_json::json!({ "error": e.to_string() })),
    }
}

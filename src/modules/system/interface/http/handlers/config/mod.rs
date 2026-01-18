use crate::core::infrastructure::audit::AuditService;
use crate::core::infrastructure::config_service::ConfigService;
use crate::core::infrastructure::database::Database;
use crate::core::utils::jwt::Claims;
use crate::core::utils::response::ApiResponse;
use crate::modules::system::interface::http::dto::config::ConfigUpdate;
use ntex::web;
use serde::Serialize;
use sqlx::Row;
use std::sync::Arc;

/// Config field metadata from DB
#[derive(Serialize, Clone)]
pub struct ConfigField {
    pub key: String,
    pub value: serde_json::Value,
    #[serde(rename = "type")]
    pub field_type: String,
    pub label: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub description: Option<String>,
    pub scope: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub min: Option<i32>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub max: Option<i32>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub options: Option<Vec<String>>,
    pub is_required: bool,
}

/// Config section grouping from DB
#[derive(Serialize, Clone)]
pub struct ConfigSection {
    pub id: String,
    pub code: String,
    pub title: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub description: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub icon: Option<String>,
    pub editable: bool,
    pub fields: Vec<ConfigField>,
}

/// Full config response
#[derive(Serialize)]
pub struct ConfigData {
    pub sections: Vec<ConfigSection>,
}

/// Get system configuration (dynamic from DB)
#[utoipa::path(
    get,
    path = "/api/v1/admin/config",
    responses(
        (status = 200, description = "System configuration fetched successfully")
    ),
    tag = "settings",
    security(
        ("bearer_auth" = [])
    )
)]
pub async fn get_config(
    db: web::types::State<Arc<Database>>,
    config_svc: web::types::State<Arc<ConfigService>>,
    req: web::HttpRequest,
) -> Result<impl web::Responder, web::Error> {
    let claims = req
        .extensions()
        .get::<Claims>()
        .cloned()
        .ok_or_else(|| web::error::ErrorUnauthorized("Missing authentication claims"))?;

    let tenant_id = claims.tenant_id;
    let is_owner = claims.is_system_owner.unwrap_or(false);

    // 1. Fetch all sections (core only, plugin_id IS NULL)
    let section_rows = sqlx::query(
        "SELECT id, code, title, description, icon, is_owner_only 
         FROM sys_config_sections 
         WHERE plugin_id IS NULL 
         ORDER BY sort_order",
    )
    .fetch_all(&db.pool)
    .await
    .unwrap_or_default();

    let mut sections: Vec<ConfigSection> = Vec::new();

    for row in section_rows {
        let section_id: uuid::Uuid = row.get("id");
        let code: String = row.get("code");
        let is_owner_only: bool = row.get("is_owner_only");

        // Skip owner-only sections for non-owners
        if is_owner_only && !is_owner {
            continue;
        }

        // 2. Fetch fields for this section
        let field_rows = sqlx::query(
            "SELECT key, label, description, field_type, scope, 
                    default_value, min_value, max_value, options, is_required, is_secret
             FROM sys_config_fields 
             WHERE section_id = $1 
             ORDER BY sort_order",
        )
        .bind(section_id)
        .fetch_all(&db.pool)
        .await
        .unwrap_or_default();

        let mut fields: Vec<ConfigField> = Vec::new();

        for field_row in field_rows {
            let key: String = field_row.get("key");
            let field_type: String = field_row.get("field_type");
            let is_secret: bool = field_row.get("is_secret");
            let default_value: Option<serde_json::Value> = field_row.get("default_value");
            let options_json: Option<serde_json::Value> = field_row.get("options");

            // 3. Get current value from sys_configs
            let current_value = if field_type == "number" {
                let default_int = default_value
                    .as_ref()
                    .and_then(|v| v.as_str())
                    .and_then(|s| s.parse::<i64>().ok())
                    .or_else(|| default_value.as_ref().and_then(|v| v.as_i64()))
                    .unwrap_or(0);
                let val = config_svc
                    .get_tenant_int(Some(tenant_id), &key, default_int)
                    .await;
                serde_json::json!(val)
            } else if field_type == "boolean" {
                let default_str = default_value
                    .as_ref()
                    .and_then(|v| v.as_str())
                    .unwrap_or("false");
                let val = config_svc
                    .get_tenant_string(Some(tenant_id), &key, default_str)
                    .await;
                serde_json::json!(val == "true")
            } else {
                // text, select, secret
                let default_str = default_value
                    .as_ref()
                    .and_then(|v| v.as_str())
                    .unwrap_or("");
                let val = config_svc
                    .get_tenant_string(Some(tenant_id), &key, default_str)
                    .await;

                // Mask secrets
                if is_secret && !val.is_empty() {
                    if val.len() > 8 {
                        serde_json::json!(format!("{}...{}", &val[..4], &val[val.len() - 4..]))
                    } else {
                        serde_json::json!("***")
                    }
                } else {
                    serde_json::json!(val)
                }
            };

            // Parse options array
            let options: Option<Vec<String>> = options_json.and_then(|v| {
                v.as_array().map(|arr| {
                    arr.iter()
                        .filter_map(|item| item.as_str().map(String::from))
                        .collect()
                })
            });

            fields.push(ConfigField {
                key,
                value: current_value,
                field_type,
                label: field_row.get("label"),
                description: field_row.get("description"),
                scope: field_row.get("scope"),
                min: field_row.get("min_value"),
                max: field_row.get("max_value"),
                options,
                is_required: field_row.get("is_required"),
            });
        }

        sections.push(ConfigSection {
            id: section_id.to_string(),
            code: code.clone(),
            title: row.get("title"),
            description: row.get("description"),
            icon: row.get("icon"),
            editable: !is_owner_only || is_owner,
            fields,
        });
    }

    let data = ConfigData { sections };

    let response = ApiResponse::ok(data, "Configuration loaded").with_meta(serde_json::json!({
        "is_owner": is_owner,
        "tenant_id": tenant_id.to_string()
    }));

    Ok(web::HttpResponse::Ok().json(&response))
}

/// Update system configuration
#[utoipa::path(
    patch,
    path = "/api/v1/admin/config",
    request_body = ConfigUpdate,
    responses(
        (status = 200, description = "System configuration updated successfully"),
        (status = 400, description = "Invalid configuration value"),
        (status = 403, description = "Permission denied"),
        (status = 500, description = "Internal server error")
    ),
    tag = "settings",
    security(
        ("bearer_auth" = [])
    )
)]
pub async fn update_config(
    body: web::types::Json<ConfigUpdate>,
    db: web::types::State<Arc<Database>>,
    config_svc: web::types::State<Arc<ConfigService>>,
    audit: web::types::State<Arc<AuditService>>,
    req: web::HttpRequest,
) -> Result<impl web::Responder, web::Error> {
    let claims = req
        .extensions()
        .get::<Claims>()
        .cloned()
        .ok_or_else(|| web::error::ErrorUnauthorized("Missing authentication claims"))?;

    let key = &body.key;
    let value = body.value.clone();
    let tenant_id = claims.tenant_id;
    let is_owner = claims.is_system_owner.unwrap_or(false);

    log::info!(
        "PATCH /config: key={} tenant={} is_owner={}",
        key,
        tenant_id,
        is_owner
    );

    // 1. Check if field exists and get its metadata
    let field_row = sqlx::query(
        "SELECT f.scope, f.min_value, f.max_value, f.field_type, s.is_owner_only
         FROM sys_config_fields f
         JOIN sys_config_sections s ON f.section_id = s.id
         WHERE f.key = $1",
    )
    .bind(key)
    .fetch_optional(&db.pool)
    .await
    .unwrap_or(None);

    let Some(field) = field_row else {
        let response = ApiResponse::<()>::not_found(&format!("Config field '{}' not found", key));
        return Ok(web::HttpResponse::NotFound().json(&response));
    };

    let is_owner_only: bool = field.get("is_owner_only");
    let scope: String = field.get("scope");
    let min_value: Option<i32> = field.get("min_value");
    let max_value: Option<i32> = field.get("max_value");
    let field_type: String = field.get("field_type");

    // 2. Check owner-only permission
    if is_owner_only && !is_owner {
        let response = ApiResponse::<()>::forbidden("Only platform owner can modify this setting");
        return Ok(web::HttpResponse::Forbidden().json(&response));
    }

    // 3. Validate min/max for numbers
    if field_type == "number"
        && let Some(num) = value.as_i64() {
            if let Some(min) = min_value
                && num < min as i64 {
                    let response =
                        ApiResponse::<()>::bad_request(&format!("Value must be at least {}", min));
                    return Ok(web::HttpResponse::BadRequest().json(&response));
                }
            if let Some(max) = max_value
                && num > max as i64 {
                    let response =
                        ApiResponse::<()>::bad_request(&format!("Value cannot exceed {}", max));
                    return Ok(web::HttpResponse::BadRequest().json(&response));
                }
        }

    // 4. Save to sys_configs
    let scope_str = if scope == "workspace" {
        "workspace"
    } else {
        "platform"
    };
    if let Err(e) = config_svc
        .set_tenant_config(Some(tenant_id), key, value.clone(), Some(scope_str))
        .await
    {
        let response = ApiResponse::<()>::internal_error(&e.message);
        return Ok(web::HttpResponse::InternalServerError().json(&response));
    }

    let _ = audit
        .log(
            &claims.sub,
            "CONFIG_UPDATE",
            Some(key),
            "SUCCESS",
            Some(value),
        )
        .await;

    let response = ApiResponse::ok(
        serde_json::json!({ "key": key, "updated": true }),
        "Configuration updated",
    );
    Ok(web::HttpResponse::Ok().json(&response))
}

use ntex::web::{self, HttpResponse};
use serde::{Deserialize, Serialize};
use std::sync::Arc;
use uuid::Uuid;
use crate::core::utils::jwt::Claims;

use crate::core::infrastructure::database::Database;
use crate::modules::auth::application::services::user_preferences::UserPreferencesService;
use crate::modules::auth::domain::user_preferences::UpdateUserPreferencesDto;

/// Request body for updating preferences
#[derive(Debug, Deserialize)]
pub struct UpdatePreferencesRequest {
    pub preferred_theme_light_id: Option<String>,
    pub preferred_theme_dark_id: Option<String>,
    pub theme_mode: Option<String>,
    pub locale: Option<String>,
    pub timezone: Option<String>,
    pub notifications_enabled: Option<bool>,
}

/// Response for user preferences
#[derive(Debug, Serialize)]
pub struct PreferencesResponse {
    pub id: String,
    pub user_id: String,
    pub preferred_theme_light_id: Option<String>,
    pub preferred_theme_dark_id: Option<String>,
    pub theme_mode: Option<String>,
    pub locale: Option<String>,
    pub timezone: Option<String>,
    pub notifications_enabled: Option<bool>,
}

/// GET /users/me/preferences - Get current user's preferences
#[utoipa::path(
    get,
    path = "/api/v1/me/preferences",
    responses(
        (status = 200, description = "User preferences retrieved", body = PreferencesResponse),
        (status = 401, description = "Not authenticated")
    ),
    tag = "users",
    security(
        ("bearer_auth" = [])
    )
)]
pub async fn get_my_preferences(
    _req: web::HttpRequest,
    db: web::types::State<Arc<Database>>,
    claims: Claims,
) -> HttpResponse {
    // Get user_id from claims
    let user_id = match Uuid::parse_str(&claims.sub) {
        Ok(id) => id,
        Err(_) => {
            return HttpResponse::Unauthorized().json(&serde_json::json!({
                "error": "Invalid user ID in token"
            }))
        }
    };

    let service = UserPreferencesService::new(db.get_ref().clone());
    
    match service.get_or_create_preferences(user_id).await {
        Ok(prefs) => HttpResponse::Ok().json(&PreferencesResponse {
            id: prefs.id.to_string(),
            user_id: prefs.user_id.to_string(),
            preferred_theme_light_id: prefs.preferred_theme_light_id,
            preferred_theme_dark_id: prefs.preferred_theme_dark_id,
            theme_mode: prefs.theme_mode,
            locale: prefs.locale,
            timezone: prefs.timezone,
            notifications_enabled: prefs.notifications_enabled,
        }),
        Err(e) => HttpResponse::InternalServerError().json(&serde_json::json!({
            "error": e
        })),
    }
}

/// PATCH /users/me/preferences - Update current user's preferences
#[utoipa::path(
    patch,
    path = "/api/v1/me/preferences",
    request_body = UpdatePreferencesRequest,
    responses(
        (status = 200, description = "User preferences updated", body = PreferencesResponse),
        (status = 401, description = "Not authenticated")
    ),
    tag = "users",
    security(
        ("bearer_auth" = [])
    )
)]
pub async fn update_my_preferences(
    _req: web::HttpRequest,
    db: web::types::State<Arc<Database>>,
    body: web::types::Json<UpdatePreferencesRequest>,
    claims: Claims,
) -> HttpResponse {
    // Get user_id from claims
    let user_id = match Uuid::parse_str(&claims.sub) {
        Ok(id) => id,
        Err(_) => {
            return HttpResponse::Unauthorized().json(&serde_json::json!({
                "error": "Invalid user ID in token"
            }))
        }
    };

    let service = UserPreferencesService::new(db.get_ref().clone());
    let dto = UpdateUserPreferencesDto {
        preferred_theme_light_id: body.preferred_theme_light_id.clone(),
        preferred_theme_dark_id: body.preferred_theme_dark_id.clone(),
        theme_mode: body.theme_mode.clone(),
        locale: body.locale.clone(),
        timezone: body.timezone.clone(),
        notifications_enabled: body.notifications_enabled,
    };

    match service.update_preferences(user_id, dto).await {
        Ok(prefs) => HttpResponse::Ok().json(&PreferencesResponse {
            id: prefs.id.to_string(),
            user_id: prefs.user_id.to_string(),
            preferred_theme_light_id: prefs.preferred_theme_light_id,
            preferred_theme_dark_id: prefs.preferred_theme_dark_id,
            theme_mode: prefs.theme_mode,
            locale: prefs.locale,
            timezone: prefs.timezone,
            notifications_enabled: prefs.notifications_enabled,
        }),
        Err(e) => HttpResponse::InternalServerError().json(&serde_json::json!({
            "error": e
        })),
    }
}

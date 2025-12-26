use ntex::web;
use std::sync::Arc;
use serde_json::json;
use crate::modules::auth::application::services::auth::AuthService;
use crate::modules::auth::domain::login::UserCredentials;
use crate::modules::auth::interface::http::dto::auth::{RefreshRequest, SetupRequest, CreateUserRequest};

/// User Login
#[utoipa::path(
    post,
    path = "/api/v1/auth/login",
    request_body = UserCredentials,
    responses(
        (status = 200, description = "Login successful", body = AuthResponse),
        (status = 401, description = "Invalid credentials")
    ),
    tag = "auth"
)]
pub async fn login(
    creds: web::types::Json<UserCredentials>,
    service: web::types::State<Arc<AuthService>>,
) -> Result<web::HttpResponse, web::Error> {
    match service.login(creds.into_inner()).await {
        Ok(auth_response) => Ok(web::HttpResponse::Ok().json(&auth_response)),
        Err(e) => Ok(web::HttpResponse::Unauthorized().json(&json!({
            "status": "error",
            "message": e.message
        }))),
    }
}

/// Refresh Session Token
#[utoipa::path(
    post,
    path = "/api/v1/auth/refresh",
    request_body = RefreshRequest,
    responses(
        (status = 200, description = "Token refreshed successfully", body = AuthResponse),
        (status = 401, description = "Invalid refresh token")
    ),
    tag = "auth"
)]
pub async fn refresh_session(
    req: web::types::Json<RefreshRequest>,
    service: web::types::State<Arc<AuthService>>,
) -> Result<web::HttpResponse, web::Error> {
    match service.refresh_session(&req.refresh_token).await {
        Ok(auth_response) => Ok(web::HttpResponse::Ok().json(&auth_response)),
        Err(e) => Ok(web::HttpResponse::Unauthorized().json(&json!({
            "status": "error",
            "message": e.message
        }))),
    }
}

/// User Logout
#[utoipa::path(
    post,
    path = "/api/v1/auth/logout",
    responses(
        (status = 200, description = "Logged out successfully")
    ),
    tag = "auth",
    security(
        ("bearer_auth" = [])
    )
)]
pub async fn logout(
    req: web::HttpRequest,
    service: web::types::State<Arc<AuthService>>,
) -> Result<web::HttpResponse, web::Error> {
    // Extract token from header
    let auth_header = req.headers().get("Authorization");
    if let Some(header) = auth_header {
        if let Ok(val) = header.to_str() {
            if val.starts_with("Bearer ") {
                let token = &val[7..];
                if let Err(e) = service.logout(token).await {
                    return Ok(web::HttpResponse::InternalServerError().json(&json!({
                        "status": "error",
                        "message": e.message
                    })));
                }
            }
        }
    }
    
    Ok(web::HttpResponse::Ok().json(&json!({
        "status": "success",
        "message": "Logged out successfully"
    })))
}

/// Check if system setup is completed
#[utoipa::path(
    get,
    path = "/api/v1/auth/setup/status",
    responses(
        (status = 200, description = "Setup status fetched")
    ),
    tag = "auth"
)]
pub async fn get_setup_status(service: web::types::State<Arc<AuthService>>) -> Result<web::HttpResponse, web::Error> {
    match service.is_setup_done().await {
        Ok(is_done) => Ok(web::HttpResponse::Ok().json(&json!({ "is_setup": is_done }))),
        Err(e) => Ok(web::HttpResponse::InternalServerError().json(&json!({
            "status": "error",
            "message": e.message
        }))),
    }
}

/// Verify Engine Secret Key (for migrations/setup)
#[utoipa::path(
    post,
    path = "/api/v1/auth/setup/verify-key",
    responses(
        (status = 200, description = "Key verified"),
        (status = 401, description = "Invalid key")
    ),
    tag = "auth",
    params(
        ("X-Engine-Secret" = String, Header, description = "Engine Secret Key")
    )
)]
pub async fn verify_engine_key(
    req: web::HttpRequest,
) -> Result<web::HttpResponse, web::Error> {
    let engine_secret = std::env::var("ENGINE_SECRET_KEY").unwrap_or_default();
    let provided_secret = req.headers().get("X-Engine-Secret")
        .and_then(|h| h.to_str().ok())
        .unwrap_or_default();

    if engine_secret.is_empty() {
        return Ok(web::HttpResponse::InternalServerError().json(&json!({
            "status": "error",
            "message": "Engine secret not configured"
        })));
    }

    if provided_secret == engine_secret {
        Ok(web::HttpResponse::Ok().json(&json!({
            "status": "success",
            "message": "Engine key verified"
        })))
    } else {
        Ok(web::HttpResponse::Unauthorized().json(&json!({
            "status": "error",
            "message": "Invalid engine secret"
        })))
    }
}

/// Check if organization slug is available (for setup)
#[utoipa::path(
    get,
    path = "/api/v1/auth/setup/check-slug",
    responses(
        (status = 200, description = "Slug availability response"),
        (status = 401, description = "Invalid engine key")
    ),
    tag = "auth",
    params(
        ("slug" = String, Query, description = "Slug to check"),
        ("X-Engine-Secret" = String, Header, description = "Engine Secret Key")
    )
)]
pub async fn check_slug_availability(
    req: web::HttpRequest,
    query: web::types::Query<std::collections::HashMap<String, String>>,
    service: web::types::State<Arc<AuthService>>,
) -> Result<web::HttpResponse, web::Error> {
    // 1. Verify Engine Secret
    let engine_secret = std::env::var("ENGINE_SECRET_KEY").unwrap_or_default();
    let provided_secret = req.headers().get("X-Engine-Secret")
        .and_then(|h| h.to_str().ok())
        .unwrap_or_default();

    if engine_secret.is_empty() || provided_secret != engine_secret {
        return Ok(web::HttpResponse::Unauthorized().json(&json!({
            "status": "error",
            "message": "Invalid engine secret"
        })));
    }

    // 2. Get slug from query params
    let slug = match query.get("slug") {
        Some(s) if !s.is_empty() => s.clone(),
        _ => return Ok(web::HttpResponse::BadRequest().json(&json!({
            "status": "error",
            "message": "Slug parameter is required"
        }))),
    };

    // 3. Check availability via service
    match service.check_slug_availability(&slug).await {
        Ok(available) => Ok(web::HttpResponse::Ok().json(&json!({
            "available": available,
            "slug": slug
        }))),
        Err(e) => Ok(web::HttpResponse::InternalServerError().json(&json!({
            "status": "error",
            "message": e.message
        })))
    }
}

/// Initialize System (SuperAdmin creation)
#[utoipa::path(
    post,
    path = "/api/v1/auth/setup",
    request_body = SetupRequest,
    responses(
        (status = 200, description = "System initialized successfully", body = AuthResponse),
        (status = 401, description = "Invalid engine secret"),
        (status = 403, description = "System already initialized")
    ),
    tag = "auth",
    params(
        ("X-Engine-Secret" = String, Header, description = "Engine Secret Key")
    )
)]
pub async fn initialize_system(
    req: web::types::Json<SetupRequest>,
    service: web::types::State<Arc<AuthService>>,
    http_req: web::HttpRequest,
) -> Result<web::HttpResponse, web::Error> {
    // 1. Verify Engine Secret
    let engine_secret = std::env::var("ENGINE_SECRET_KEY").unwrap_or_default();
    let provided_secret = http_req.headers().get("X-Engine-Secret")
        .and_then(|h| h.to_str().ok())
        .unwrap_or_default();

    if engine_secret.is_empty() || provided_secret != engine_secret {
        return Ok(web::HttpResponse::Unauthorized().json(&json!({
            "status": "error",
            "message": "Invalid engine secret"
        })));
    }

    // 2. Initialize System
    match service.initialize_system(req.into_inner()).await {
        Ok(response) => Ok(web::HttpResponse::Ok().json(&response)),
        Err(e) => {
            let mut status = if e.code == 403 {
                web::HttpResponse::Forbidden()
            } else {
                web::HttpResponse::InternalServerError()
            };
            Ok(status.json(&json!({
                "status": "error",
                "message": e.message
            })))
        }
    }
}

pub async fn register(
    req: web::types::Json<SetupRequest>,
    service: web::types::State<Arc<AuthService>>,
    http_req: web::HttpRequest,
) -> Result<web::HttpResponse, web::Error> {
    // 1. Verify Engine Secret (Duplicated Logic for Safety)
    let engine_secret = std::env::var("ENGINE_SECRET_KEY").unwrap_or_default();
    let provided_secret = http_req.headers().get("X-Engine-Secret")
        .and_then(|h| h.to_str().ok())
        .unwrap_or_default();

    if engine_secret.is_empty() || provided_secret != engine_secret {
        return Ok(web::HttpResponse::Unauthorized().json(&json!({
            "status": "error",
            "message": "Invalid engine secret"
        })));
    }

    // 2. Initialize System
    match service.initialize_system(req.into_inner()).await {
        Ok(response) => Ok(web::HttpResponse::Ok().json(&response)),
        Err(e) => {
            let mut status = if e.code == 403 {
                web::HttpResponse::Forbidden()
            } else {
                web::HttpResponse::InternalServerError()
            };
            Ok(status.json(&json!({
                "status": "error",
                "message": e.message
            })))
        }
    }
}

/// Create a new user (Admin)
#[utoipa::path(
    post,
    path = "/api/v1/auth/users",
    request_body = CreateUserRequest,
    responses(
        (status = 201, description = "User created successfully"),
        (status = 400, description = "Invalid input"),
        (status = 403, description = "Insufficient permissions")
    ),
    tag = "auth",
    security(
        ("bearer_auth" = [])
    )
)]
pub async fn create_user(
    req: web::types::Json<CreateUserRequest>,
    service: web::types::State<Arc<AuthService>>,
) -> Result<web::HttpResponse, web::Error> {
    match service.create_user(req.into_inner()).await {
        Ok(id) => Ok(web::HttpResponse::Created().json(&json!({
            "status": "success",
            "message": "User created successfully",
            "data": { "id": id }
        }))),
        Err(e) => {
            let mut status = match e.code {
                400 => web::HttpResponse::BadRequest(),
                401 => web::HttpResponse::Unauthorized(),
                403 => web::HttpResponse::Forbidden(),
                404 => web::HttpResponse::NotFound(),
                _ => web::HttpResponse::InternalServerError(),
            };
            Ok(status.json(&json!({
                "status": "error",
                "message": e.message
            })))
        }
    }
}

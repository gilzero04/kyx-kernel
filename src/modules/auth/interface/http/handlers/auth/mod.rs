use ntex::web;
use std::sync::Arc;
use serde_json::json;
use crate::modules::auth::application::services::auth::AuthService;
use crate::modules::auth::domain::login::UserCredentials;
use crate::modules::auth::interface::http::dto::auth::{RefreshRequest, SetupRequest, CreateUserRequest, SignupRequest};
#[allow(unused_imports)]
use crate::modules::auth::interface::http::dto::auth::{AuthResponse, SessionInfo, AdminSessionInfo};
use crate::core::infrastructure::config_service::ConfigService;
use uuid::Uuid;

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
    req: web::HttpRequest,
    creds: web::types::Json<UserCredentials>,
    service: web::types::State<Arc<AuthService>>,
) -> Result<web::HttpResponse, web::Error> {
    let ip = if let Some(real_ip) = req.headers().get("X-Real-IP")
        .and_then(|h| h.to_str().ok()) {
        real_ip.trim().to_string()
    } else if let Some(forwarded) = req.headers().get("X-Forwarded-For")
        .and_then(|h| h.to_str().ok())
        .and_then(|list| list.split(',').next()) {
        forwarded.trim().to_string()
    } else {
        req.connection_info().remote().unwrap_or("unknown").to_string()
    };

    let ua = req.headers().get("User-Agent")
        .and_then(|h| h.to_str().ok())
        .unwrap_or("unknown")
        .to_string();

    match service.login(creds.into_inner(), ip, ua).await {
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
pub async fn get_setup_status(
    service: web::types::State<Arc<AuthService>>,
    config: web::types::State<Arc<ConfigService>>,
) -> Result<web::HttpResponse, web::Error> {
    match service.is_setup_done().await {
        Ok(is_done) => {
            // Fetch branding data for theme switching
            let theme_light_id = config.get_string("theme_light_id", "").await;
            let theme_dark_id = config.get_string("theme_dark_id", "").await;
            let app_name = config.get_string("branding_app_name", "").await;
            let splash_text = config.get_string("branding_splash_text", "").await;
            let splash_init_text = config.get_string("branding_splash_init_text", "").await;
            
            Ok(web::HttpResponse::Ok().json(&json!({ 
                "is_setup": is_done,
                "branding": {
                    "app_name": if app_name.is_empty() { None } else { Some(app_name) },
                    "theme_light_id": theme_light_id,
                    "theme_dark_id": theme_dark_id,
                    "splash": {
                        "text": if splash_text.is_empty() { None } else { Some(splash_text) },
                        "subtext": if splash_init_text.is_empty() { None } else { Some(splash_init_text) }
                    }
                }
            })))
        },
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

/// Public Signup (Create Tenant & Admin)
#[utoipa::path(
    post,
    path = "/api/v1/auth/signup",
    request_body = SignupRequest,
    responses(
        (status = 200, description = "Signup successful", body = AuthResponse),
        (status = 400, description = "Invalid input or slug taken")
    ),
    tag = "auth"
)]
pub async fn signup(
    req: web::types::Json<SignupRequest>,
    service: web::types::State<Arc<AuthService>>,
) -> Result<web::HttpResponse, web::Error> {
    match service.signup(req.into_inner()).await {
        Ok(response) => Ok(web::HttpResponse::Ok().json(&response)),
        Err(e) => {
            let mut status = match e.code {
                400 => web::HttpResponse::BadRequest(),
                _ => web::HttpResponse::InternalServerError(),
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

/// List Active Sessions
#[utoipa::path(
    get,
    path = "/api/v1/auth/sessions",
    responses(
        (status = 200, description = "List of active sessions", body = [SessionInfo])
    ),
    tag = "auth",
    security(
        ("bearer_auth" = [])
    )
)]
pub async fn list_sessions(
    claims: crate::core::utils::jwt::Claims,
    service: web::types::State<Arc<AuthService>>,
) -> Result<web::HttpResponse, web::Error> {
    let user_id = Uuid::parse_str(&claims.sub).map_err(|_| web::error::ErrorBadRequest("Invalid user ID"))?;
    match service.list_sessions(&user_id).await {
        Ok(sessions) => Ok(web::HttpResponse::Ok().json(&sessions)),
        Err(e) => Ok(web::HttpResponse::InternalServerError().json(&json!({
            "status": "error",
            "message": e.message
        }))),
    }
}

/// Revoke a Session
#[utoipa::path(
    delete,
    path = "/api/v1/auth/sessions/{sid}",
    responses(
        (status = 200, description = "Session revoked"),
        (status = 404, description = "Session not found")
    ),
    tag = "auth",
    security(
        ("bearer_auth" = [])
    )
)]
pub async fn revoke_session(
    claims: crate::core::utils::jwt::Claims,
    path: web::types::Path<(String,)>,
    service: web::types::State<Arc<AuthService>>,
) -> Result<web::HttpResponse, web::Error> {
    let user_id = Uuid::parse_str(&claims.sub).map_err(|_| web::error::ErrorBadRequest("Invalid user ID"))?;
    let (sid,) = path.into_inner();
    
    match service.revoke_session(&user_id, &sid).await {
        Ok(_) => Ok(web::HttpResponse::Ok().json(&json!({
            "status": "success",
            "message": "Session revoked"
        }))),
        Err(e) => Ok(web::HttpResponse::InternalServerError().json(&json!({
            "status": "error",
            "message": e.message
        }))),
    }
}

/// List All Active Sessions (Admin)
#[utoipa::path(
    get,
    path = "/api/v1/auth/admin/sessions",
    responses(
        (status = 200, description = "List of all active sessions", body = [AdminSessionInfo])
    ),
    tag = "auth",
    security(
        ("bearer_auth" = [])
    )
)]
pub async fn admin_list_sessions(
    claims: crate::core::utils::jwt::Claims, // Ensure authenticated
    service: web::types::State<Arc<AuthService>>,
) -> Result<web::HttpResponse, web::Error> {
    // Note: RBAC middleware should handle permission check ("system:manage")
    match service.list_all_sessions(claims.sid).await {
        Ok(sessions) => Ok(web::HttpResponse::Ok().json(&sessions)),
        Err(e) => Ok(web::HttpResponse::InternalServerError().json(&json!({
            "status": "error",
            "message": e.message
        }))),
    }
}

/// Revoke Any Session (Admin)
#[utoipa::path(
    delete,
    path = "/api/v1/auth/admin/sessions/{user_id}/{sid}",
    responses(
        (status = 200, description = "Session revoked"),
        (status = 404, description = "Session not found")
    ),
    tag = "auth",
    security(
        ("bearer_auth" = [])
    )
)]
pub async fn admin_revoke_session_handler(
    _claims: crate::core::utils::jwt::Claims,
    path: web::types::Path<(Uuid, String)>,
    service: web::types::State<Arc<AuthService>>,
) -> Result<web::HttpResponse, web::Error> {
    let (user_id, sid) = path.into_inner();
    
    match service.admin_revoke_session(user_id, sid).await {
        Ok(_) => Ok(web::HttpResponse::Ok().json(&json!({
            "status": "success",
            "message": "Session revoked by admin"
        }))),
        Err(e) => Ok(web::HttpResponse::InternalServerError().json(&json!({
            "status": "error",
            "message": e.message
        }))),
    }
}

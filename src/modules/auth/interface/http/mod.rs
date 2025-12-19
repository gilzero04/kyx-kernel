use ntex::web;
use crate::modules::auth::application::login_service::AuthService;
use crate::modules::auth::domain::login::UserCredentials;
use std::sync::Arc;
use serde_json::json;
use serde::Deserialize;

#[derive(Debug, Deserialize)]
pub struct RefreshRequest {
    pub refresh_token: String,
}


#[web::post("/login")]
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

#[web::post("/refresh")]
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

#[web::post("/logout")]
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

#[web::get("/setup/status")]
pub async fn get_setup_status(service: web::types::State<Arc<AuthService>>) -> Result<web::HttpResponse, web::Error> {
    match service.is_setup_done().await {
        Ok(is_done) => Ok(web::HttpResponse::Ok().json(&json!({ "is_setup": is_done }))),
        Err(e) => Ok(web::HttpResponse::InternalServerError().json(&json!({
            "status": "error",
            "message": e.message
        }))),
    }
}

#[web::post("/setup/verify-key")]
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

#[web::post("/setup")]
pub async fn initialize_system(
    req: web::types::Json<crate::modules::auth::application::login_service::SetupRequest>,
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

#[web::post("/register")]
pub async fn register(
    req: web::types::Json<crate::modules::auth::application::login_service::SetupRequest>,
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

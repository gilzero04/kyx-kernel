use std::sync::Arc;
use ntex::service::{Middleware, Service, ServiceCtx};
use ntex::web;
use crate::core::utils::jwt::JwtService;
use crate::core::infrastructure::audit::AuditService;
use crate::core::infrastructure::redis::Redis;

#[derive(Clone)]
pub struct RequirePermission {
    pub permission: String,
    pub system_only: bool,
    pub jwt_service: Arc<JwtService>,
    pub audit_service: Arc<AuditService>,
    pub redis_service: Option<Arc<Redis>>,
}


impl RequirePermission {
    pub fn new(permission: impl Into<String>, jwt_service: Arc<JwtService>, audit_service: Arc<AuditService>) -> Self {
        Self { 
            permission: permission.into(), 
            system_only: false, 
            jwt_service, 
            audit_service,
            redis_service: None,
        }
    }

    pub fn system(permission: impl Into<String>, jwt_service: Arc<JwtService>, audit_service: Arc<AuditService>) -> Self {
        Self { 
            permission: permission.into(), 
            system_only: true, 
            jwt_service, 
            audit_service,
            redis_service: None,
        }
    }

    pub fn with_redis(mut self, redis: Arc<Redis>) -> Self {
        self.redis_service = Some(redis);
        self
    }

    pub fn with_redis_opt(mut self, redis: Option<Arc<Redis>>) -> Self {
        self.redis_service = redis;
        self
    }
}

impl<S> Middleware<S> for RequirePermission {
    type Service = RequirePermissionMiddleware<S>;

    fn create(&self, service: S) -> Self::Service {
        RequirePermissionMiddleware {
            service,
            permission: self.permission.clone(),
            system_only: self.system_only,
            jwt_service: self.jwt_service.clone(),
            audit_service: self.audit_service.clone(),
            redis_service: self.redis_service.clone(),
        }
    }
}

pub struct RequirePermissionMiddleware<S> {
    service: S,
    permission: String,
    system_only: bool,
    jwt_service: Arc<JwtService>,
    audit_service: Arc<AuditService>,
    redis_service: Option<Arc<Redis>>,
}

impl<S, Err> Service<web::WebRequest<Err>> for RequirePermissionMiddleware<S>
where
    S: Service<web::WebRequest<Err>, Response = web::WebResponse, Error = web::Error>,
{
    type Response = web::WebResponse;
    type Error = web::Error;

    async fn call(&self, req: web::WebRequest<Err>, ctx: ServiceCtx<'_, Self>) -> Result<Self::Response, Self::Error> {
        let auth_header = req.headers().get("Authorization");
        
        let token = match auth_header {
            Some(h) => h.to_str().ok().and_then(|v| v.strip_prefix("Bearer ")),
            None => None,
        };

        if let Some(t) = token {
            match self.jwt_service.verify_token(t) {
                Ok(claims) => {
                    // Check if session exists in Redis if sid is present
                    if let (Some(redis), Some(sid)) = (&self.redis_service, &claims.sid) {
                        let key = format!("auth:session:{}:{}", claims.sub, sid);
                        let mut conn = redis.get_connection();
                        let exists: i32 = redis::cmd("EXISTS")
                            .arg(&key)
                            .query_async(&mut conn)
                            .await
                            .unwrap_or(0);
                        
                        if exists == 0 {
                            println!("[RequirePermission] Session revoked for user: {}, sid: {}", claims.sub, sid);
                            return Ok(req.into_response(
                                web::HttpResponse::Unauthorized().finish()
                            ));
                        } else {
                            // println!("[RequirePermission] Session valid for user: {}, sid: {}", claims.sub, sid);
                        }
                    }

                    // 1. Mandatory System Owner check if system_only is true
                    let is_owner = claims.is_system_owner.unwrap_or(false);
                    if self.system_only && !is_owner {
                         return Ok(req.into_response(
                            web::HttpResponse::Forbidden().finish()
                        ));
                    }

                    // 2. Permission check
                    if self.permission.is_empty() || is_owner || claims.permissions.contains(&"system:manage".to_string()) || claims.permissions.contains(&self.permission) {
                        // Authorized - attach claims to request extensions and continue
                        req.extensions_mut().insert(claims);
                        return ctx.call(&self.service, req).await;
                    } else {
                        // Log unauthorized attempt
                        let audit = self.audit_service.clone();
                        let sub = claims.sub.clone();
                        let path = req.path().to_string();
                        let required_perm = self.permission.clone();
                        
                        tokio::spawn(async move {
                            let _ = audit.log(
                                &sub, 
                                "UNAUTHORIZED_ACCESS", 
                                Some(&path), 
                                "FAILURE", 
                                Some(format!("Missing permission: {}", required_perm).into())
                            ).await;
                        });
                        
                        return Ok(req.into_response(
                            web::HttpResponse::Forbidden().finish()
                        ));
                    }
                }
                Err(_) => {
                    // Invalid token
                    return Ok(req.into_response(
                        web::HttpResponse::Unauthorized().finish()
                    ));
                }
            }
        }

        Ok(req.into_response(
            web::HttpResponse::Unauthorized().finish()
        ))
    }
}

#[derive(Clone)]
pub struct RequirePlatform {
    pub jwt_service: Arc<JwtService>,
}

impl RequirePlatform {
    #[allow(dead_code)]
    pub fn new(jwt_service: Arc<JwtService>) -> Self {
        Self { jwt_service }
    }
}

impl<S> Middleware<S> for RequirePlatform {
    type Service = RequirePlatformMiddleware<S>;
    fn create(&self, service: S) -> Self::Service {
        RequirePlatformMiddleware { service, jwt_service: self.jwt_service.clone() }
    }
}

pub struct RequirePlatformMiddleware<S> {
    service: S,
    jwt_service: Arc<JwtService>,
}

impl<S, Err> Service<web::WebRequest<Err>> for RequirePlatformMiddleware<S>
where
    S: Service<web::WebRequest<Err>, Response = web::WebResponse, Error = web::Error>,
{
    type Response = web::WebResponse;
    type Error = web::Error;

    async fn call(&self, req: web::WebRequest<Err>, ctx: ServiceCtx<'_, Self>) -> Result<Self::Response, Self::Error> {
        let auth_header = req.headers().get("Authorization");
        let token = auth_header.and_then(|h| h.to_str().ok()).and_then(|v| v.strip_prefix("Bearer "));

        if let Some(t) = token {
            if let Ok(claims) = self.jwt_service.verify_token(t) {
                if claims.is_system_owner.unwrap_or(false) {
                    req.extensions_mut().insert(claims);
                    return ctx.call(&self.service, req).await;
                }
            }
        }

        Ok(req.into_response(web::HttpResponse::Forbidden().finish()))
    }
}

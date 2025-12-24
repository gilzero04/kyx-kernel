use std::sync::Arc;
use ntex::service::{Middleware, Service, ServiceCtx};
use ntex::web;
use crate::core::utils::jwt::JwtService;
use crate::core::infrastructure::audit::AuditService;

#[derive(Clone)]
pub struct RequirePermission {
    pub permission: String,
    pub jwt_service: Arc<JwtService>,
    pub audit_service: Arc<AuditService>,
}

impl RequirePermission {
    pub fn new(permission: impl Into<String>, jwt_service: Arc<JwtService>, audit_service: Arc<AuditService>) -> Self {
        Self { permission: permission.into(), jwt_service, audit_service }
    }
}

impl<S> Middleware<S> for RequirePermission {
    type Service = RequirePermissionMiddleware<S>;

    fn create(&self, service: S) -> Self::Service {
        RequirePermissionMiddleware {
            service,
            permission: self.permission.clone(),
            jwt_service: self.jwt_service.clone(),
            audit_service: self.audit_service.clone(),
        }
    }
}

pub struct RequirePermissionMiddleware<S> {
    service: S,
    permission: String,
    jwt_service: Arc<JwtService>,
    audit_service: Arc<AuditService>,
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
                    // Check if user has the required permission
                    if claims.permissions.contains(&self.permission) {
                        // Authorized - continue to next service
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

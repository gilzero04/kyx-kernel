use std::sync::Arc;
use ntex::service::{Middleware, Service, ServiceCtx};
use ntex::web;
use crate::core::utils::jwt::JwtService;
use crate::core::infrastructure::audit::AuditService;
use crate::core::domain::auth::UserRole;

pub struct RequireRole {
    pub min_role: UserRole,
    pub jwt_service: Arc<JwtService>,
    pub audit_service: Arc<AuditService>,
}

impl RequireRole {
    pub fn new(min_role: UserRole, jwt_service: Arc<JwtService>, audit_service: Arc<AuditService>) -> Self {
        Self { min_role, jwt_service, audit_service }
    }
}

impl<S> Middleware<S> for RequireRole {
    type Service = RequireRoleMiddleware<S>;

    fn create(&self, service: S) -> Self::Service {
        RequireRoleMiddleware {
            service,
            min_role: self.min_role.clone(),
            jwt_service: self.jwt_service.clone(),
            audit_service: self.audit_service.clone(),
        }
    }
}

pub struct RequireRoleMiddleware<S> {
    service: S,
    min_role: UserRole,
    jwt_service: Arc<JwtService>,
    audit_service: Arc<AuditService>,
}

impl<S, Err> Service<web::WebRequest<Err>> for RequireRoleMiddleware<S>
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
                    let user_role = UserRole::from_str(&claims.role);
                    if user_role >= self.min_role {
                        // Authorized - continue to next service
                        return ctx.call(&self.service, req).await;
                    } else {
                        // Log unauthorized attempt
                        let audit = self.audit_service.clone();
                        let sub = claims.sub.clone();
                        let path = req.path().to_string();
                        
                        // We can't easily spark a tokio task here if we want to be ultra-clean, 
                        // but let's just do it background style for now.
                        tokio::spawn(async move {
                            let _ = audit.log(&sub, "UNAUTHORIZED_ACCESS", Some(&path), "FAILURE", None).await;
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

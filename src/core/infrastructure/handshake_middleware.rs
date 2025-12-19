use std::sync::Arc;
use ntex::service::{Middleware, Service, ServiceCtx};
use ntex::web;

pub struct EngineHandshake {
    secret_key: String,
}

impl EngineHandshake {
    pub fn new(secret_key: String) -> Self {
        Self { secret_key }
    }
}

impl<S> Middleware<S> for EngineHandshake {
    type Service = EngineHandshakeMiddleware<S>;

    fn create(&self, service: S) -> Self::Service {
        EngineHandshakeMiddleware {
            service,
            secret_key: self.secret_key.clone(),
        }
    }
}

pub struct EngineHandshakeMiddleware<S> {
    service: S,
    secret_key: String,
}

impl<S, Err> Service<web::WebRequest<Err>> for EngineHandshakeMiddleware<S>
where
    S: Service<web::WebRequest<Err>, Response = web::WebResponse, Error = web::Error>,
{
    type Response = web::WebResponse;
    type Error = web::Error;

    async fn call(&self, req: web::WebRequest<Err>, ctx: ServiceCtx<'_, Self>) -> Result<Self::Response, Self::Error> {
        let handshake_header = req.headers().get("X-Engine-Secret");

        let is_valid = match handshake_header {
            Some(h) => h.to_str().map(|v| v == self.secret_key).unwrap_or(false),
            None => false,
        };

        if !is_valid {
            return Ok(req.into_response(
                web::HttpResponse::Forbidden()
                    .json(&serde_json::json!({
                        "error": "Handshake failed: Invalid Engine Secret"
                    }))
            ));
        }

        ctx.call(&self.service, req).await
    }
}

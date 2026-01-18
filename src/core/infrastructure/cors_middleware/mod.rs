use crate::core::infrastructure::cors::CorsManager;
use ntex::http::header::{
    ACCESS_CONTROL_ALLOW_CREDENTIALS, ACCESS_CONTROL_ALLOW_HEADERS, ACCESS_CONTROL_ALLOW_METHODS,
    ACCESS_CONTROL_ALLOW_ORIGIN, ACCESS_CONTROL_MAX_AGE, HeaderValue, ORIGIN, VARY,
};
use ntex::http::{Method, StatusCode};
use ntex::service::{Middleware, Service, ServiceCtx};
use ntex::web;
use std::sync::Arc;

pub struct DynamicCors {
    manager: Arc<CorsManager>,
}

impl DynamicCors {
    pub fn new(manager: Arc<CorsManager>) -> Self {
        Self { manager }
    }
}

impl<S> Middleware<S> for DynamicCors {
    type Service = DynamicCorsMiddleware<S>;

    fn create(&self, service: S) -> Self::Service {
        DynamicCorsMiddleware {
            service,
            manager: self.manager.clone(),
        }
    }
}

pub struct DynamicCorsMiddleware<S> {
    service: S,
    manager: Arc<CorsManager>,
}

impl<S, Err> Service<web::WebRequest<Err>> for DynamicCorsMiddleware<S>
where
    S: Service<web::WebRequest<Err>, Response = web::WebResponse, Error = web::Error>,
{
    type Response = web::WebResponse;
    type Error = web::Error;

    async fn call(
        &self,
        req: web::WebRequest<Err>,
        ctx: ServiceCtx<'_, Self>,
    ) -> Result<Self::Response, Self::Error> {
        let origin = req
            .headers()
            .get(ORIGIN)
            .and_then(|h| h.to_str().ok())
            .map(|s| s.to_string());

        let is_preflight = req.method() == Method::OPTIONS;

        // Check if origin is allowed
        let allowed = if let Some(ref origin_str) = origin {
            let is_dev = std::env::var("ENVIRONMENT").unwrap_or_default() == "local";
            let is_local_net = origin_str.starts_with("http://192.168.")
                || origin_str.starts_with("http://10.")
                || origin_str.starts_with("http://172.")
                || origin_str.contains("localhost")
                || origin_str.contains("127.0.0.1");

            if is_dev && is_local_net {
                true
            } else {
                self.manager.is_origin_allowed(origin_str).await
            }
        } else {
            true // Allow same-origin/server-side fetches
        };

        // Handle OPTIONS preflight requests
        if is_preflight && allowed {
            if let Some(ref origin_str) = origin {
                if let Ok(hv) = HeaderValue::from_str(origin_str) {
                    let mut res =
                        req.into_response(web::HttpResponse::build(StatusCode::OK).finish());
                    res.headers_mut().insert(ACCESS_CONTROL_ALLOW_ORIGIN, hv);
                    res.headers_mut().insert(
                        ACCESS_CONTROL_ALLOW_METHODS,
                        HeaderValue::from_static("GET, POST, PUT, DELETE, OPTIONS, PATCH"),
                    );
                    res.headers_mut().insert(
                        ACCESS_CONTROL_ALLOW_HEADERS,
                        HeaderValue::from_static(
                            "Content-Type, Authorization, X-Engine-Secret, X-Requested-With",
                        ),
                    );
                    res.headers_mut().insert(
                        ACCESS_CONTROL_ALLOW_CREDENTIALS,
                        HeaderValue::from_static("true"),
                    );
                    res.headers_mut()
                        .insert(ACCESS_CONTROL_MAX_AGE, HeaderValue::from_static("86400"));
                    res.headers_mut()
                        .append(VARY, HeaderValue::from_static("Origin"));
                    return Ok(res);
                }
            }
        }

        // Call the actual service
        let res = ctx.call(&self.service, req).await?;

        // Add CORS headers to response if origin is allowed
        if let Some(origin_str) = origin {
            if allowed {
                if let Ok(hv) = HeaderValue::from_str(&origin_str) {
                    let mut res = res;
                    res.headers_mut().insert(ACCESS_CONTROL_ALLOW_ORIGIN, hv);
                    res.headers_mut().insert(
                        ACCESS_CONTROL_ALLOW_METHODS,
                        HeaderValue::from_static("GET, POST, PUT, DELETE, OPTIONS, PATCH"),
                    );
                    res.headers_mut().insert(
                        ACCESS_CONTROL_ALLOW_HEADERS,
                        HeaderValue::from_static(
                            "Content-Type, Authorization, X-Engine-Secret, X-Requested-With",
                        ),
                    );
                    res.headers_mut().insert(
                        ACCESS_CONTROL_ALLOW_CREDENTIALS,
                        HeaderValue::from_static("true"),
                    );
                    res.headers_mut()
                        .append(VARY, HeaderValue::from_static("Origin"));
                    return Ok(res);
                }
            }
        }

        Ok(res)
    }
}

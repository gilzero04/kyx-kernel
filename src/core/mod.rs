use ntex::web;

pub mod actor;
pub mod application;
pub mod bootstrap;
pub mod domain;
pub mod event;
pub mod infrastructure;
pub mod utils;

/// Every module in `src/modules` must implement this trait or
/// provide a function to register its routes following this pattern.
pub trait AppModule: Send + Sync {
    /// Unique name of the module
    #[allow(dead_code)]
    fn name(&self) -> &str;

    /// Entry point for registering module routes into ntex.
    /// Returns an error if the module fails to initialize its specific infrastructure.
    fn try_configure(&self, config: &mut web::ServiceConfig) -> Result<(), AppError>;
}

/// Global Error types for the entire Kernel
#[derive(Debug, serde::Serialize, serde::Deserialize)]
pub struct AppError {
    pub code: u16,
    pub message: String,
}

pub type AppResult<T> = Result<T, AppError>;

impl std::fmt::Display for AppError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "AppError(code={}, message={})", self.code, self.message)
    }
}

impl std::error::Error for AppError {}

impl AppError {
    #[allow(dead_code)]
    pub fn new(code: u16, message: String) -> Self {
        Self { code, message }
    }

    pub fn bad_request(message: impl Into<String>) -> Self {
        Self {
            code: 400,
            message: message.into(),
        }
    }

    pub fn internal_server_error(message: impl Into<String>) -> Self {
        Self {
            code: 500,
            message: message.into(),
        }
    }

    #[allow(dead_code)]
    pub fn forbidden(message: impl Into<String>) -> Self {
        Self {
            code: 403,
            message: message.into(),
        }
    }

    #[allow(dead_code)]
    pub fn not_found(message: impl Into<String>) -> Self {
        Self {
            code: 404,
            message: message.into(),
        }
    }

    #[allow(dead_code)]
    pub fn unauthorized(message: impl Into<String>) -> Self {
        Self {
            code: 401,
            message: message.into(),
        }
    }
}

impl From<sqlx::Error> for AppError {
    fn from(err: sqlx::Error) -> Self {
        Self {
            code: 500,
            message: format!("Database error: {}", err),
        }
    }
}
impl web::error::WebResponseError for AppError {
    fn error_response(&self, _: &web::HttpRequest) -> web::HttpResponse {
        web::HttpResponse::build(
            ntex::http::StatusCode::from_u16(self.code)
                .unwrap_or(ntex::http::StatusCode::INTERNAL_SERVER_ERROR),
        )
        .json(&serde_json::json!({
            "code": self.code,
            "message": self.message
        }))
    }
}

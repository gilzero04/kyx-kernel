use ntex::web;

pub mod domain;
pub mod application;
pub mod infrastructure;
pub mod utils;

/// Every module in `src/modules` must implement this trait or
/// provide a function to register its routes following this pattern.
pub trait AppModule: Send + Sync {
    /// Unique name of the module
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

impl std::fmt::Display for AppError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "AppError(code={}, message={})", self.code, self.message)
    }
}

impl std::error::Error for AppError {}

impl From<sqlx::Error> for AppError {
    fn from(err: sqlx::Error) -> Self {
        Self {
            code: 500,
            message: format!("Database error: {}", err),
        }
    }
}

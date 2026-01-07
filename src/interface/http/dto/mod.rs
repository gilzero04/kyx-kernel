use serde::Serialize;

#[derive(Debug, Serialize)]
#[allow(dead_code)]
pub struct ApiResponse<T> {
    pub success: bool,
    pub data: Option<T>,
    pub error: Option<ApiError>,
}

#[derive(Debug, Serialize)]
#[allow(dead_code)]
pub struct ApiError {
    pub code: u16,
    pub message: String,
    pub details: Option<serde_json::Value>,
}

// success and error helpers removed until used in controllers

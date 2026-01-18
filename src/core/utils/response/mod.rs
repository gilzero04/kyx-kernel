use chrono::Utc;
use serde::Serialize;

/// Standard API Response wrapper for all endpoints
#[derive(Serialize)]
pub struct ApiResponse<T: Serialize> {
    pub success: bool,
    pub status: u16,
    pub message: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub data: Option<T>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub error: Option<ApiError>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub pagination: Option<Pagination>,
    pub meta: ResponseMeta,
}

/// Pagination information for list endpoints
#[derive(Serialize, Clone)]
pub struct Pagination {
    pub page: u32,
    pub per_page: u32,
    pub total_items: u64,
    pub total_pages: u32,
    pub has_next: bool,
    pub has_prev: bool,
}

impl Pagination {
    pub fn new(page: u32, per_page: u32, total_items: u64) -> Self {
        let total_pages = ((total_items as f64) / (per_page as f64)).ceil() as u32;
        Self {
            page,
            per_page,
            total_items,
            total_pages,
            has_next: page < total_pages,
            has_prev: page > 1,
        }
    }
}

/// Error details for failed requests
#[derive(Serialize, Clone)]
pub struct ApiError {
    pub code: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub field: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub details: Option<String>,
}

impl ApiError {
    pub fn new(code: &str) -> Self {
        Self {
            code: code.to_string(),
            field: None,
            details: None,
        }
    }

    pub fn with_field(mut self, field: &str) -> Self {
        self.field = Some(field.to_string());
        self
    }

    pub fn with_details(mut self, details: &str) -> Self {
        self.details = Some(details.to_string());
        self
    }
}

/// Metadata included in every response
#[derive(Serialize, Clone)]
pub struct ResponseMeta {
    pub timestamp: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub request_id: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub response_time_ms: Option<u64>,
    #[serde(skip_serializing_if = "Option::is_none")]
    #[serde(flatten)]
    pub extra: Option<serde_json::Value>,
}

impl Default for ResponseMeta {
    fn default() -> Self {
        Self {
            timestamp: Utc::now().to_rfc3339(),
            request_id: None,
            response_time_ms: None,
            extra: None,
        }
    }
}

impl ResponseMeta {
    pub fn with_extra(mut self, extra: serde_json::Value) -> Self {
        self.extra = Some(extra);
        self
    }
}

impl<T: Serialize> ApiResponse<T> {
    /// Create a successful response with data
    pub fn ok(data: T, message: &str) -> Self {
        Self {
            success: true,
            status: 200,
            message: message.to_string(),
            data: Some(data),
            error: None,
            pagination: None,
            meta: ResponseMeta::default(),
        }
    }

    /// Create a successful response with custom status
    pub fn success(status: u16, data: T, message: &str) -> Self {
        Self {
            success: true,
            status,
            message: message.to_string(),
            data: Some(data),
            error: None,
            pagination: None,
            meta: ResponseMeta::default(),
        }
    }

    /// Create a 201 Created response
    pub fn created(data: T, message: &str) -> Self {
        Self::success(201, data, message)
    }

    /// Create a paginated response
    pub fn paginated(data: T, pagination: Pagination, message: &str) -> Self {
        Self {
            success: true,
            status: 200,
            message: message.to_string(),
            data: Some(data),
            error: None,
            pagination: Some(pagination),
            meta: ResponseMeta::default(),
        }
    }

    /// Add extra metadata to response
    pub fn with_meta(mut self, extra: serde_json::Value) -> Self {
        self.meta.extra = Some(extra);
        self
    }
}

/// Error response without data (uses unit type)
impl ApiResponse<()> {
    /// Create an error response
    pub fn error(status: u16, code: &str, message: &str) -> Self {
        Self {
            success: false,
            status,
            message: message.to_string(),
            data: None,
            error: Some(ApiError::new(code)),
            pagination: None,
            meta: ResponseMeta::default(),
        }
    }

    /// Create an error response with details
    pub fn error_with_details(status: u16, code: &str, message: &str, details: &str) -> Self {
        Self {
            success: false,
            status,
            message: message.to_string(),
            data: None,
            error: Some(ApiError::new(code).with_details(details)),
            pagination: None,
            meta: ResponseMeta::default(),
        }
    }

    /// 400 Bad Request
    pub fn bad_request(message: &str) -> Self {
        Self::error(400, "VALIDATION_ERROR", message)
    }

    /// 401 Unauthorized
    pub fn unauthorized(message: &str) -> Self {
        Self::error(401, "UNAUTHORIZED", message)
    }

    /// 403 Forbidden
    pub fn forbidden(message: &str) -> Self {
        Self::error(403, "PERMISSION_DENIED", message)
    }

    /// 404 Not Found
    pub fn not_found(message: &str) -> Self {
        Self::error(404, "NOT_FOUND", message)
    }

    /// 409 Conflict
    pub fn conflict(message: &str) -> Self {
        Self::error(409, "CONFLICT", message)
    }

    /// 429 Too Many Requests
    pub fn rate_limited(message: &str) -> Self {
        Self::error(429, "RATE_LIMITED", message)
    }

    /// 500 Internal Server Error
    pub fn internal_error(message: &str) -> Self {
        Self::error(500, "INTERNAL_ERROR", message)
    }
}

// Common error codes
pub mod error_codes {
    pub const VALIDATION_ERROR: &str = "VALIDATION_ERROR";
    pub const UNAUTHORIZED: &str = "UNAUTHORIZED";
    pub const PERMISSION_DENIED: &str = "PERMISSION_DENIED";
    pub const NOT_FOUND: &str = "NOT_FOUND";
    pub const CONFLICT: &str = "CONFLICT";
    pub const RATE_LIMITED: &str = "RATE_LIMITED";
    pub const INTERNAL_ERROR: &str = "INTERNAL_ERROR";
    pub const OWNER_ONLY: &str = "OWNER_ONLY";
}

use serde::Deserialize;
use utoipa::{IntoParams, ToSchema};

#[derive(Debug, Deserialize, ToSchema, IntoParams)]
pub struct UsersQuery {
    pub page: Option<i64>,
    pub limit: Option<i64>,
    pub sort: Option<String>,
    pub search: Option<String>,
}

#[derive(Deserialize, ToSchema)]
pub struct UpdateUserRequest {
    pub full_name: Option<String>,
    pub is_active: Option<bool>,
    pub role_slug: Option<String>,
    pub tenant_id: Option<sqlx::types::Uuid>,
}

#[derive(Deserialize, ToSchema)]
pub struct AdminResetPasswordRequest {
    pub new_password: String,
}

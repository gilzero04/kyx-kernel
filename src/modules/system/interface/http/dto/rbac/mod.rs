use serde::Deserialize;
use utoipa::ToSchema;

#[derive(Debug, Deserialize, ToSchema)]
pub struct CreateRoleRequest {
    pub name: String,
    pub slug: Option<String>,
    pub code: Option<String>,
    pub description: Option<String>,
    pub is_active: Option<bool>,
}

#[derive(Debug, Deserialize, ToSchema)]
pub struct UpdateRoleRequest {
    pub name: Option<String>,
    pub code: Option<String>,
    pub description: Option<String>,
    pub is_active: Option<bool>,
}

#[derive(Debug, Deserialize, ToSchema)]
pub struct CreatePermissionRequest {
    pub name: String,
    pub slug: Option<String>,
    pub code: Option<String>,
    pub description: Option<String>,
    pub is_active: Option<bool>,
}

#[derive(Debug, Deserialize, ToSchema)]
pub struct UpdatePermissionRequest {
    pub name: Option<String>,
    pub code: Option<String>,
    pub description: Option<String>,
    pub is_active: Option<bool>,
}

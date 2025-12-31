use serde::{Deserialize, Serialize};
use uuid::Uuid;
use utoipa::ToSchema;

#[derive(Debug, Serialize, Deserialize, ToSchema)]
pub struct UserInfo {
    pub id: Uuid,
    pub email: String,
    pub full_name: Option<String>,
    pub role: String,
    pub permissions: Vec<String>,
    pub tenant_type: Option<String>,
    pub avatar_url: Option<String>,
    pub cover_url: Option<String>,
    pub images: Vec<UserImageInfo>,
}

#[derive(Debug, Serialize, Deserialize, ToSchema)]
pub struct UserImageInfo {
    pub id: Uuid,
    pub url: String,
    pub image_type: String, // 'avatar', 'cover'
    pub is_primary: bool,
    pub created_at: String,
}

#[derive(Debug, Serialize, Deserialize, ToSchema)]
pub struct AuthResponse {
    pub access_token: String,
    pub refresh_token: String,
    pub user: UserInfo,
}

#[derive(Debug, Deserialize, ToSchema)]
pub struct SetupRequest {
    pub email: String,
    pub password: String,
    pub full_name: String,
    pub org_name: String,
    pub org_slug: Option<String>,
    pub app_name: Option<String>,
    pub platform_type: String, // 'single' | 'multi'
    pub primary_color: Option<String>,
    pub secondary_color: Option<String>,
    pub accent_color: Option<String>,
}

#[derive(Debug, Deserialize, ToSchema)]
pub struct CreateUserRequest {
    pub email: String,
    pub password: String,
    pub full_name: String,
    pub tenant_id: Uuid,
    pub role_slug: String,
}

#[derive(Debug, Deserialize, ToSchema)]
pub struct RefreshRequest {
    pub refresh_token: String,
}

#[derive(Debug, Deserialize, ToSchema)]
pub struct SignupRequest {
    pub email: String,
    pub password: String,
    pub full_name: String,
    pub org_name: String,
    pub org_slug: String,
    pub plan_type: String, // 'standard' | 'business'
    pub parent_id: Option<Uuid>,
}

#[derive(Debug, Serialize, Deserialize, ToSchema)]
pub struct SessionInfo {
    pub sid: String,
    pub ip: String,
    pub user_agent: String,
    pub created_at: i64,
    pub expires_at: i64,
}

#[derive(Debug, Serialize, Deserialize, ToSchema)]
pub struct AdminSessionInfo {
    pub sid: String,
    pub user_id: Uuid,
    pub email: String,
    pub full_name: Option<String>,
    pub ip: String,
    pub user_agent: String,
    pub created_at: i64,
    pub expires_at: i64,
    pub is_current: bool,
}

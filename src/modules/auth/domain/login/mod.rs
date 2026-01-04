use serde::{Deserialize, Serialize};
use uuid::Uuid;
use utoipa::ToSchema;

#[derive(Debug, Clone, Serialize, Deserialize, ToSchema)]
#[allow(dead_code)]
pub struct User {
    pub id: Uuid,
    pub username: String,
    pub password_hash: String,
    pub role: UserRole,
}

use crate::core::domain::auth::UserRole;

#[derive(Debug, Clone, Serialize, Deserialize, ToSchema)]
pub struct UserCredentials {
    pub username: String,
    pub password: String,
}

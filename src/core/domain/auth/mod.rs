use serde::{Deserialize, Serialize};
use utoipa::ToSchema;
use uuid::Uuid;

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, PartialOrd, ToSchema)]
pub enum UserRole {
    Viewer = 0,
    Operator = 1,
    Admin = 2,
    SuperAdmin = 3,
}

#[derive(Debug, Clone, Serialize, Deserialize, ToSchema)]
#[allow(dead_code)]
pub struct Permission {
    pub id: Uuid,
    pub slug: String,
    pub name: String,
}

#[derive(Debug, Clone, Serialize, Deserialize, ToSchema)]
#[allow(dead_code)]
pub struct RoleDescriptor {
    pub id: Uuid,
    pub slug: String,
    pub name: String,
    pub permissions: Vec<Permission>,
}

impl UserRole {
    pub fn from_str(role: &str) -> Self {
        match role.to_lowercase().as_str() {
            "superadmin" => Self::SuperAdmin,
            "admin" => Self::Admin,
            "operator" => Self::Operator,
            _ => Self::Viewer,
        }
    }

    #[allow(dead_code)]
    pub fn as_str(&self) -> &str {
        match self {
            Self::SuperAdmin => "SuperAdmin",
            Self::Admin => "Admin",
            Self::Operator => "Operator",
            Self::Viewer => "Viewer",
        }
    }
}

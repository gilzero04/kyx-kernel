use super::entity::{Role, Permission};
use anyhow::Result;
use async_trait::async_trait;
use uuid::Uuid;

#[derive(Debug)]
pub struct CreateRoleCmd {
    pub code: Option<String>,
    pub slug: String,
    pub name: String,
    pub description: Option<String>,
    pub is_active: Option<bool>,
}

#[derive(Debug)]
pub struct UpdateRoleCmd {
    pub name: Option<String>,
    pub code: Option<String>,
    pub description: Option<String>,
    pub is_active: Option<bool>,
}

#[derive(Debug)]
pub struct CreatePermissionCmd {
    pub code: Option<String>,
    pub slug: String,
    pub name: String,
    pub description: Option<String>,
    pub is_active: Option<bool>,
}

#[derive(Debug)]
pub struct UpdatePermissionCmd {
    pub name: Option<String>,
    pub code: Option<String>,
    pub description: Option<String>,
    pub is_active: Option<bool>,
}

#[async_trait]
pub trait RbacRepository: Send + Sync {
    // Roles
    async fn list_roles(&self) -> Result<Vec<Role>>;
    async fn create_role(&self, cmd: CreateRoleCmd) -> Result<Role>;
    async fn update_role(&self, id: Uuid, cmd: UpdateRoleCmd) -> Result<Option<Role>>;
    async fn delete_role(&self, id: Uuid) -> Result<bool>;

    // Permissions
    async fn list_permissions(&self) -> Result<Vec<Permission>>;
    async fn create_permission(&self, cmd: CreatePermissionCmd) -> Result<Permission>;
    async fn update_permission(&self, id: Uuid, cmd: UpdatePermissionCmd) -> Result<Option<Permission>>;
    async fn delete_permission(&self, id: Uuid) -> Result<bool>;
}

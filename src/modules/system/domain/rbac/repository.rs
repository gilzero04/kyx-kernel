use super::entity::{Permission, Role};
use anyhow::Result;
use async_trait::async_trait;
use uuid::Uuid;

#[derive(Debug)]
pub struct CreateRoleCmd {
    pub tenant_id: Option<Uuid>,
    pub code: Option<String>,
    pub slug: String,
    pub name: String,
    pub description: Option<String>,
    pub is_active: Option<bool>,
}

#[derive(Debug)]
pub struct UpdateRoleCmd {
    pub name: Option<String>,
    pub code: Option<Option<String>>,
    pub description: Option<Option<String>>,
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
    pub code: Option<Option<String>>,
    pub description: Option<Option<String>>,
    pub is_active: Option<bool>,
}

#[async_trait]
pub trait RbacRepository: Send + Sync {
    // Roles
    async fn list_roles(
        &self,
        tenant_id: Option<Uuid>,
        actor_tenant_id: Option<Uuid>,
        show_all: bool,
    ) -> Result<Vec<Role>>;
    async fn create_role(&self, cmd: CreateRoleCmd, actor_tenant_id: Option<Uuid>) -> Result<Role>;
    async fn update_role(
        &self,
        id: Uuid,
        cmd: UpdateRoleCmd,
        actor_tenant_id: Option<Uuid>,
    ) -> Result<Option<Role>>;
    async fn delete_role(&self, id: Uuid, actor_tenant_id: Option<Uuid>) -> Result<bool>;
    async fn get_role_permissions(
        &self,
        role_id: Uuid,
        actor_tenant_id: Option<Uuid>,
    ) -> Result<Vec<Permission>>;
    async fn update_role_permissions(
        &self,
        role_id: Uuid,
        permission_ids: Vec<Uuid>,
        actor_tenant_id: Option<Uuid>,
    ) -> Result<()>;

    // Permissions
    async fn list_permissions(
        &self,
        tenant_id: Option<Uuid>,
        actor_tenant_id: Option<Uuid>,
    ) -> Result<Vec<Permission>>;
    async fn create_permission(&self, cmd: CreatePermissionCmd) -> Result<Permission>;
    async fn update_permission(
        &self,
        id: Uuid,
        cmd: UpdatePermissionCmd,
    ) -> Result<Option<Permission>>;
    async fn delete_permission(&self, id: Uuid) -> Result<bool>;
}

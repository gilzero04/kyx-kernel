use std::sync::Arc;
use crate::modules::system::domain::rbac::{
    RbacRepository, Role, Permission, 
    CreateRoleCmd, UpdateRoleCmd, 
    CreatePermissionCmd, UpdatePermissionCmd
};
use crate::core::AppError;
use uuid::Uuid;

mod role_actions;
mod permission_actions;

pub struct RbacService {
    repo: Arc<dyn RbacRepository>,
}

impl RbacService {
    pub fn new(repo: Arc<dyn RbacRepository>) -> Self {
        Self { repo }
    }

    // === Permissions ===
    pub async fn list_permissions(&self, actor_tenant_id: Option<Uuid>) -> Result<Vec<Permission>, AppError> {
        permission_actions::list(&self.repo, actor_tenant_id).await
    }
    
    pub async fn create_permission(&self, cmd: CreatePermissionCmd, actor_tenant_id: Option<Uuid>) -> Result<Permission, AppError> {
        permission_actions::create(&self.repo, cmd, actor_tenant_id).await
    }

    pub async fn update_permission(&self, id: Uuid, cmd: UpdatePermissionCmd, actor_tenant_id: Option<Uuid>) -> Result<Permission, AppError> {
        permission_actions::update(&self.repo, id, cmd, actor_tenant_id).await
    }

    pub async fn delete_permission(&self, id: Uuid, actor_tenant_id: Option<Uuid>) -> Result<(), AppError> {
        permission_actions::delete(&self.repo, id, actor_tenant_id).await
    }

    // === Roles ===
    pub async fn list_roles(&self, tenant_id: Option<Uuid>, actor_tenant_id: Option<Uuid>, show_all: bool) -> Result<Vec<Role>, AppError> {
        role_actions::list(&self.repo, tenant_id, actor_tenant_id, show_all).await
    }

    pub async fn create_role(&self, cmd: CreateRoleCmd, actor_tenant_id: Option<Uuid>) -> Result<Role, AppError> {
        role_actions::create(&self.repo, cmd, actor_tenant_id).await
    }

    pub async fn update_role(&self, id: Uuid, cmd: UpdateRoleCmd, actor_tenant_id: Option<Uuid>) -> Result<Role, AppError> {
        role_actions::update(&self.repo, id, cmd, actor_tenant_id).await
    }

    pub async fn delete_role(&self, id: Uuid, actor_tenant_id: Option<Uuid>) -> Result<(), AppError> {
        role_actions::delete(&self.repo, id, actor_tenant_id).await
    }

    pub async fn get_role_permissions(&self, role_id: Uuid, actor_tenant_id: Option<Uuid>) -> Result<Vec<Permission>, AppError> {
        role_actions::get_permissions(&self.repo, role_id, actor_tenant_id).await
    }

    pub async fn update_role_permissions(&self, role_id: Uuid, permission_ids: Vec<Uuid>, actor_tenant_id: Option<Uuid>) -> Result<(), AppError> {
        role_actions::update_permissions(&self.repo, role_id, permission_ids, actor_tenant_id).await
    }
}

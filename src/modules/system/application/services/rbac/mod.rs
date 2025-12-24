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
    pub async fn list_permissions(&self) -> Result<Vec<Permission>, AppError> {
        permission_actions::list(&self.repo).await
    }
    
    pub async fn create_permission(&self, cmd: CreatePermissionCmd) -> Result<Permission, AppError> {
        permission_actions::create(&self.repo, cmd).await
    }

    pub async fn update_permission(&self, id: Uuid, cmd: UpdatePermissionCmd) -> Result<Permission, AppError> {
        permission_actions::update(&self.repo, id, cmd).await
    }

    pub async fn delete_permission(&self, id: Uuid) -> Result<(), AppError> {
        permission_actions::delete(&self.repo, id).await
    }

    // === Roles ===
    pub async fn list_roles(&self) -> Result<Vec<Role>, AppError> {
        role_actions::list(&self.repo).await
    }

    pub async fn create_role(&self, cmd: CreateRoleCmd) -> Result<Role, AppError> {
        role_actions::create(&self.repo, cmd).await
    }

    pub async fn update_role(&self, id: Uuid, cmd: UpdateRoleCmd) -> Result<Role, AppError> {
        role_actions::update(&self.repo, id, cmd).await
    }

    pub async fn delete_role(&self, id: Uuid) -> Result<(), AppError> {
        role_actions::delete(&self.repo, id).await
    }
}

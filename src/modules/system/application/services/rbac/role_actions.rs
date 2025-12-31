use std::sync::Arc;
use crate::modules::system::domain::rbac::{RbacRepository, Role, Permission, CreateRoleCmd, UpdateRoleCmd};
use crate::core::AppError;
use uuid::Uuid;

pub async fn list(repo: &Arc<dyn RbacRepository>, tenant_id: Option<Uuid>, actor_tenant_id: Option<Uuid>, show_all: bool) -> Result<Vec<Role>, AppError> {
    repo.list_roles(tenant_id, actor_tenant_id, show_all).await.map_err(|e| AppError { code: 500, message: e.to_string() })
}

pub async fn create(repo: &Arc<dyn RbacRepository>, cmd: CreateRoleCmd, actor_tenant_id: Option<Uuid>) -> Result<Role, AppError> {
    repo.create_role(cmd, actor_tenant_id).await.map_err(|e| AppError { code: 500, message: e.to_string() })
}

pub async fn update(repo: &Arc<dyn RbacRepository>, id: Uuid, cmd: UpdateRoleCmd, actor_tenant_id: Option<Uuid>) -> Result<Role, AppError> {
    repo.update_role(id, cmd, actor_tenant_id).await
        .map_err(|e| AppError { code: 500, message: e.to_string() })?
        .ok_or_else(|| AppError { code: 404, message: "Role not found".into() })
}

pub async fn delete(repo: &Arc<dyn RbacRepository>, id: Uuid, actor_tenant_id: Option<Uuid>) -> Result<(), AppError> {
    repo.delete_role(id, actor_tenant_id).await.map_err(|e| AppError { code: 500, message: e.to_string() })
        .and_then(|deleted| if deleted { Ok(()) } else { Err(AppError { code: 404, message: "Role not found".into() }) })
}

pub async fn get_permissions(repo: &Arc<dyn RbacRepository>, role_id: Uuid, actor_tenant_id: Option<Uuid>) -> Result<Vec<Permission>, AppError> {
    repo.get_role_permissions(role_id, actor_tenant_id).await.map_err(|e| AppError { code: 500, message: e.to_string() })
}

pub async fn update_permissions(repo: &Arc<dyn RbacRepository>, role_id: Uuid, permission_ids: Vec<Uuid>, actor_tenant_id: Option<Uuid>) -> Result<(), AppError> {
    repo.update_role_permissions(role_id, permission_ids, actor_tenant_id).await.map_err(|e| AppError { code: 500, message: e.to_string() })
}


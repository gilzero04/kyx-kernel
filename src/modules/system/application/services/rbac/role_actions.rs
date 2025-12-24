use std::sync::Arc;
use crate::modules::system::domain::rbac::{RbacRepository, Role, CreateRoleCmd, UpdateRoleCmd};
use crate::core::AppError;
use uuid::Uuid;

pub async fn list(repo: &Arc<dyn RbacRepository>) -> Result<Vec<Role>, AppError> {
    repo.list_roles().await.map_err(|e| AppError { code: 500, message: e.to_string() })
}

pub async fn create(repo: &Arc<dyn RbacRepository>, cmd: CreateRoleCmd) -> Result<Role, AppError> {
    repo.create_role(cmd).await.map_err(|e| AppError { code: 500, message: e.to_string() })
}

pub async fn update(repo: &Arc<dyn RbacRepository>, id: Uuid, cmd: UpdateRoleCmd) -> Result<Role, AppError> {
    repo.update_role(id, cmd).await
        .map_err(|e| AppError { code: 500, message: e.to_string() })?
        .ok_or_else(|| AppError { code: 404, message: "Role not found".into() })
}

pub async fn delete(repo: &Arc<dyn RbacRepository>, id: Uuid) -> Result<(), AppError> {
    repo.delete_role(id).await.map_err(|e| AppError { code: 500, message: e.to_string() })
        .and_then(|deleted| if deleted { Ok(()) } else { Err(AppError { code: 404, message: "Role not found".into() }) })
}

use std::sync::Arc;
use crate::modules::system::domain::rbac::{RbacRepository, Permission, CreatePermissionCmd, UpdatePermissionCmd};
use crate::core::AppError;
use uuid::Uuid;

pub async fn list(repo: &Arc<dyn RbacRepository>) -> Result<Vec<Permission>, AppError> {
    repo.list_permissions().await.map_err(|e| AppError { code: 500, message: e.to_string() })
}

pub async fn create(repo: &Arc<dyn RbacRepository>, cmd: CreatePermissionCmd) -> Result<Permission, AppError> {
    repo.create_permission(cmd).await.map_err(|e| AppError { code: 500, message: e.to_string() })
}

pub async fn update(repo: &Arc<dyn RbacRepository>, id: Uuid, cmd: UpdatePermissionCmd) -> Result<Permission, AppError> {
    repo.update_permission(id, cmd).await
        .map_err(|e| AppError { code: 500, message: e.to_string() })?
        .ok_or_else(|| AppError { code: 404, message: "Permission not found".into() })
}

pub async fn delete(repo: &Arc<dyn RbacRepository>, id: Uuid) -> Result<(), AppError> {
    repo.delete_permission(id).await.map_err(|e| AppError { code: 500, message: e.to_string() })
        .and_then(|deleted| if deleted { Ok(()) } else { Err(AppError { code: 404, message: "Permission not found".into() }) })
}

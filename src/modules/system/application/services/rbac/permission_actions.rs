use crate::core::AppError;
use crate::modules::system::domain::rbac::{
    CreatePermissionCmd, Permission, RbacRepository, UpdatePermissionCmd,
};
use std::sync::Arc;
use uuid::Uuid;

pub async fn list(
    repo: &Arc<dyn RbacRepository>,
    actor_tenant_id: Option<Uuid>,
) -> Result<Vec<Permission>, AppError> {
    repo.list_permissions(None, actor_tenant_id)
        .await
        .map_err(|e| AppError {
            code: 500,
            message: e.to_string(),
        })
}

pub async fn create(
    repo: &Arc<dyn RbacRepository>,
    cmd: CreatePermissionCmd,
    actor_tenant_id: Option<Uuid>,
) -> Result<Permission, AppError> {
    if actor_tenant_id.is_some() {
        return Err(AppError {
            code: 403,
            message: "Only system owners can manage global permissions".into(),
        });
    }
    repo.create_permission(cmd).await.map_err(|e| AppError {
        code: 500,
        message: e.to_string(),
    })
}

pub async fn update(
    repo: &Arc<dyn RbacRepository>,
    id: Uuid,
    cmd: UpdatePermissionCmd,
    actor_tenant_id: Option<Uuid>,
) -> Result<Permission, AppError> {
    if actor_tenant_id.is_some() {
        return Err(AppError {
            code: 403,
            message: "Only system owners can manage global permissions".into(),
        });
    }
    repo.update_permission(id, cmd)
        .await
        .map_err(|e| AppError {
            code: 500,
            message: e.to_string(),
        })?
        .ok_or_else(|| AppError {
            code: 404,
            message: "Permission not found".into(),
        })
}

pub async fn delete(
    repo: &Arc<dyn RbacRepository>,
    id: Uuid,
    actor_tenant_id: Option<Uuid>,
) -> Result<(), AppError> {
    if actor_tenant_id.is_some() {
        return Err(AppError {
            code: 403,
            message: "Only system owners can manage global permissions".into(),
        });
    }
    repo.delete_permission(id)
        .await
        .map_err(|e| AppError {
            code: 500,
            message: e.to_string(),
        })
        .and_then(|deleted| {
            if deleted {
                Ok(())
            } else {
                Err(AppError {
                    code: 404,
                    message: "Permission not found".into(),
                })
            }
        })
}

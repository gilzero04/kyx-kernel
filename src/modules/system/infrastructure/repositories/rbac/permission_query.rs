use crate::modules::system::domain::rbac::{Permission, CreatePermissionCmd, UpdatePermissionCmd};
use crate::core::infrastructure::database::Database;
use anyhow::{Result, anyhow};
use std::sync::Arc;
use uuid::Uuid;

pub async fn list(pool: &Arc<Database>) -> Result<Vec<Permission>> {
    let perms = sqlx::query_as::<_, Permission>(
        "SELECT id, code, slug, name, description, is_active, created_at, updated_at 
         FROM sys_permissions WHERE deleted_at IS NULL ORDER BY sort_order ASC, name ASC"
    )
    .fetch_all(&pool.pool)
    .await
    .map_err(|e| anyhow!("Failed to list permissions: {}", e))?;
    Ok(perms)
}

pub async fn create(pool: &Arc<Database>, cmd: CreatePermissionCmd) -> Result<Permission> {
    let perm = sqlx::query_as::<_, Permission>(
        "INSERT INTO sys_permissions (code, slug, name, description, is_active) VALUES ($1, $2, $3, $4, $5) RETURNING *"
    )
    .bind(cmd.code)
    .bind(cmd.slug)
    .bind(cmd.name)
    .bind(cmd.description)
    .bind(cmd.is_active.unwrap_or(true))
    .fetch_one(&pool.pool)
    .await
    .map_err(|e| anyhow!("Failed to create permission: {}", e))?;
    Ok(perm)
}

pub async fn update(pool: &Arc<Database>, id: Uuid, cmd: UpdatePermissionCmd) -> Result<Option<Permission>> {
    let perm = sqlx::query_as::<_, Permission>(
        "UPDATE sys_permissions SET 
         name = COALESCE($1, name), 
         code = COALESCE($2, code),
         description = COALESCE($3, description),
         is_active = COALESCE($4, is_active),
         updated_at = NOW()
         WHERE id = $5 AND deleted_at IS NULL
         RETURNING *"
    )
    .bind(cmd.name)
    .bind(cmd.code)
    .bind(cmd.description)
    .bind(cmd.is_active)
    .bind(id)
    .fetch_optional(&pool.pool)
    .await
    .map_err(|e| anyhow!("Failed to update permission: {}", e))?;
    Ok(perm)
}

pub async fn delete(pool: &Arc<Database>, id: Uuid) -> Result<bool> {
    let result = sqlx::query("UPDATE sys_permissions SET deleted_at = NOW() WHERE id = $1 AND deleted_at IS NULL")
        .bind(id)
        .execute(&pool.pool)
        .await
        .map_err(|e| anyhow!("Failed to delete permission: {}", e))?;
    Ok(result.rows_affected() > 0)
}

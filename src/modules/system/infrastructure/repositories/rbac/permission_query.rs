use crate::modules::system::domain::rbac::{Permission, CreatePermissionCmd, UpdatePermissionCmd};
use crate::core::infrastructure::database::Database;
use anyhow::{Result, anyhow};
use std::sync::Arc;
use uuid::Uuid;

pub async fn list(pool: &Arc<Database>, tenant_id: Option<Uuid>, actor_tenant_id: Option<Uuid>) -> Result<Vec<Permission>> {
    // If actor is present, they ONLY see permissions delegated to their tenant
    let final_tid = actor_tenant_id.or(tenant_id);

    let perms = if let Some(tid) = final_tid {
        sqlx::query_as::<_, Permission>(
            "SELECT p.id, p.code, p.slug, p.name, p.description, p.is_system, p.is_active, p.created_at, p.updated_at 
             FROM sys_permissions p
             JOIN sys_tenant_permissions tp ON p.id = tp.permission_id
             WHERE tp.tenant_id = $1 AND p.deleted_at IS NULL 
             ORDER BY p.sort_order ASC, p.name ASC"
        )
        .bind(tid)
        .fetch_all(&pool.pool)
        .await
        .map_err(|e| anyhow!("Failed to list scoped permissions: {}", e))?
    } else {
        // System owner sees all
        sqlx::query_as::<_, Permission>(
            "SELECT id, code, slug, name, description, is_system, is_active, created_at, updated_at 
             FROM sys_permissions WHERE deleted_at IS NULL ORDER BY sort_order ASC, name ASC"
        )
        .fetch_all(&pool.pool)
        .await
        .map_err(|e| anyhow!("Failed to list permissions: {}", e))?
    };
    Ok(perms)
}

pub async fn create(pool: &Arc<Database>, cmd: CreatePermissionCmd) -> Result<Permission> {
    let perm = sqlx::query_as::<_, Permission>(
        "INSERT INTO sys_permissions (code, slug, name, description, is_active) 
         VALUES ($1, $2, $3, $4, $5) 
         RETURNING id, code, slug, name, description, is_system, is_active, created_at, updated_at"
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
    let (code_val, code_present) = match cmd.code {
        Some(inner) => (inner, true),
        None => (None, false),
    };
    let (desc_val, desc_present) = match cmd.description {
        Some(inner) => (inner, true),
        None => (None, false),
    };

    let perm = sqlx::query_as::<_, Permission>(
        "UPDATE sys_permissions SET 
         name = COALESCE($1, name), 
         code = CASE WHEN $2 THEN NULLIF($3, '') ELSE code END,
         description = CASE WHEN $4 THEN NULLIF($5, '') ELSE description END,
         is_active = COALESCE($6, is_active),
         updated_at = NOW()
         WHERE id = $7 AND deleted_at IS NULL
         RETURNING id, code, slug, name, description, is_system, is_active, created_at, updated_at"
    )
    .bind(cmd.name)
    .bind(code_present)
    .bind(code_val)
    .bind(desc_present)
    .bind(desc_val)
    .bind(cmd.is_active)
    .bind(id)
    .fetch_optional(&pool.pool)
    .await
    .map_err(|e| anyhow!("Failed to update permission: {}", e))?;
    Ok(perm)
}

pub async fn delete(pool: &Arc<Database>, id: Uuid) -> Result<bool> {
    let result = sqlx::query("UPDATE sys_permissions SET is_active = FALSE, deleted_at = NOW() WHERE id = $1 AND deleted_at IS NULL")
        .bind(id)
        .execute(&pool.pool)
        .await
        .map_err(|e| anyhow!("Failed to delete permission: {}", e))?;
    Ok(result.rows_affected() > 0)
}

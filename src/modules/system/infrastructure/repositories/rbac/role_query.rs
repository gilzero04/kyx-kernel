use crate::modules::system::domain::rbac::{Role, Permission, CreateRoleCmd, UpdateRoleCmd};
use crate::core::infrastructure::database::Database;
use anyhow::{Result, anyhow};
use std::sync::Arc;
use uuid::Uuid;

pub async fn list(pool: &Arc<Database>, tenant_id: Option<Uuid>, actor_tenant_id: Option<Uuid>, show_all: bool) -> Result<Vec<Role>> {
    // If actor is NOT system owner, they are strictly restricted to their own tenant_id.
    // If actor IS system owner (actor_tenant_id IS None):
    // - if show_all is true, return everything.
    // - if tenant_id is Some, return that tenant's roles.
    // - otherwise (default), return only global roles (tenant_id IS NULL).
    
    let (final_tid, override_all) = match actor_tenant_id {
        Some(aid) => (Some(aid), false), // Restricted
        None => (tenant_id, show_all),    // System owner can query specific or all
    };

     let roles = sqlx::query_as::<_, Role>(
        "SELECT r.id, r.tenant_id, t.name as tenant_name, r.code, r.slug, r.name, r.description, r.is_active, r.sort_order, r.max_members, r.created_at, r.updated_at,
         (SELECT COUNT(*) FROM sys_role_permissions rp WHERE rp.role_id = r.id) as permission_count,
         (SELECT COUNT(*) FROM auth_memberships m WHERE m.role_id = r.id AND m.deleted_at IS NULL) as member_count
         FROM sys_roles r
         LEFT JOIN auth_tenants t ON r.tenant_id = t.id
         WHERE (
            ($1::uuid IS NULL AND r.tenant_id IS NULL AND $2::boolean IS FALSE) OR 
            (r.tenant_id = $1 AND $1::uuid IS NOT NULL) OR 
            ($2::boolean IS TRUE)
         )
         AND r.deleted_at IS NULL 
         ORDER BY r.sort_order ASC, r.name ASC"
    )
    .bind(final_tid)
    .bind(override_all)
    .fetch_all(&pool.pool)
    .await
    .map_err(|e| anyhow!("Failed to list roles: {}", e))?;
    Ok(roles)
}

pub async fn create(pool: &Arc<Database>, cmd: CreateRoleCmd, actor_tenant_id: Option<Uuid>) -> Result<Role> {
    // If actor is present, force their tenant_id
    let target_tenant = actor_tenant_id.or(cmd.tenant_id);

    let role = sqlx::query_as::<_, Role>(
        "INSERT INTO sys_roles (tenant_id, code, slug, name, description, is_active) 
         VALUES ($1, $2, $3, $4, $5, $6) 
         RETURNING id, tenant_id, (SELECT name FROM auth_tenants WHERE id = $1) as tenant_name, code, slug, name, description, is_active, sort_order, max_members, created_at, updated_at,
         (0::BIGINT) as permission_count, (0::BIGINT) as member_count"
    )
    .bind(target_tenant)
    .bind(cmd.code)
    .bind(cmd.slug)
    .bind(cmd.name)
    .bind(cmd.description)
    .bind(cmd.is_active.unwrap_or(true))
    .fetch_one(&pool.pool)
    .await
    .map_err(|e| anyhow!("Failed to create role: {}", e))?;
    Ok(role)
}

pub async fn update(pool: &Arc<Database>, id: Uuid, cmd: UpdateRoleCmd, actor_tenant_id: Option<Uuid>) -> Result<Option<Role>> {
    let (code_val, code_present) = match cmd.code {
        Some(inner) => (inner, true),
        None => (None, false),
    };
    let (desc_val, desc_present) = match cmd.description {
        Some(inner) => (inner, true),
        None => (None, false),
    };

    let role = sqlx::query_as::<_, Role>(
        "UPDATE sys_roles SET 
         name = COALESCE($1, name), 
         code = CASE WHEN $2 THEN NULLIF($3, '') ELSE code END,
         description = CASE WHEN $4 THEN NULLIF($5, '') ELSE description END,
         is_active = COALESCE($6, is_active),
         updated_at = NOW()
         WHERE id = $7 AND deleted_at IS NULL 
         AND (tenant_id = $8 OR $8 IS NULL)
         RETURNING id, tenant_id, (SELECT name FROM auth_tenants WHERE id = tenant_id) as tenant_name, code, slug, name, description, is_active, sort_order, max_members, created_at, updated_at,
         (SELECT COUNT(*) FROM sys_role_permissions rp WHERE rp.role_id = sys_roles.id) as permission_count,
         (SELECT COUNT(*) FROM auth_memberships m WHERE m.role_id = sys_roles.id AND m.deleted_at IS NULL) as member_count"
    )
    .bind(cmd.name)
    .bind(code_present)
    .bind(code_val)
    .bind(desc_present)
    .bind(desc_val)
    .bind(cmd.is_active)
    .bind(id)
    .bind(actor_tenant_id)
    .fetch_optional(&pool.pool)
    .await
    .map_err(|e| anyhow!("Failed to update role: {}", e))?;
    Ok(role)
}

pub async fn delete(pool: &Arc<Database>, id: Uuid, actor_tenant_id: Option<Uuid>) -> Result<bool> {
    let result = sqlx::query("UPDATE sys_roles SET is_active = FALSE, deleted_at = NOW() WHERE id = $1 AND deleted_at IS NULL AND (tenant_id = $2 OR $2 IS NULL)")
        .bind(id)
        .bind(actor_tenant_id)
        .execute(&pool.pool)
        .await
        .map_err(|e| anyhow!("Failed to delete role: {}", e))?;
    Ok(result.rows_affected() > 0)
}

pub async fn get_permissions(pool: &Arc<Database>, role_id: Uuid, actor_tenant_id: Option<Uuid>) -> Result<Vec<Permission>> {
    let permissions = sqlx::query_as::<_, Permission>(
        "SELECT p.id, p.code, p.slug, p.name, p.description, p.is_system, p.is_active, p.created_at, p.updated_at 
         FROM sys_permissions p
         JOIN sys_role_permissions rp ON p.id = rp.permission_id
         JOIN sys_roles r ON rp.role_id = r.id
         WHERE rp.role_id = $1 AND p.deleted_at IS NULL
         AND (r.tenant_id = $2 OR $2 IS NULL)"
    )
    .bind(role_id)
    .bind(actor_tenant_id)
    .fetch_all(&pool.pool)
    .await
    .map_err(|e| anyhow!("Failed to get role permissions: {}", e))?;
    Ok(permissions)
}

pub async fn update_permissions(pool: &Arc<Database>, role_id: Uuid, permission_ids: Vec<Uuid>, actor_tenant_id: Option<Uuid>) -> Result<()> {
    // 1. Verify role access
    let role_exists = sqlx::query_scalar::<_, i64>("SELECT COUNT(*) FROM sys_roles WHERE id = $1 AND (tenant_id = $2 OR $2 IS NULL) AND deleted_at IS NULL")
        .bind(role_id)
        .bind(actor_tenant_id)
        .fetch_one(&pool.pool)
        .await?;
    
    if role_exists == 0 {
        return Err(anyhow!("Role not found or access denied"));
    }

    let mut tx = pool.pool.begin().await.map_err(|e| anyhow!("Failed to begin transaction: {}", e))?;
    
    // Delete existing
    sqlx::query("DELETE FROM sys_role_permissions WHERE role_id = $1")
        .bind(role_id)
        .execute(&mut *tx)
        .await
        .map_err(|e| anyhow!("Failed to delete existing mappings: {}", e))?;
        
    // Insert new
    for perm_id in permission_ids {
        sqlx::query("INSERT INTO sys_role_permissions (role_id, permission_id) VALUES ($1, $2)")
            .bind(role_id)
            .bind(perm_id)
            .execute(&mut *tx)
            .await
            .map_err(|e| anyhow!("Failed to insert mapping: {}", e))?;
    }
    
    tx.commit().await.map_err(|e| anyhow!("Failed to commit transaction: {}", e))?;
    Ok(())
}

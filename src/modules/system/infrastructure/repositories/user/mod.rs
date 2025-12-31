use crate::modules::system::domain::user::{TenantMemberCount, UserRepository, PaginatedUsers, UserFilter};
use crate::core::infrastructure::database::Database;
use async_trait::async_trait;
use anyhow::{Result, anyhow};
use std::sync::Arc;
use uuid::Uuid;

mod list_query;

pub struct PostgresUserRepository {
    pool: Arc<Database>,
}

impl PostgresUserRepository {
    pub fn new(pool: Arc<Database>) -> Self {
        Self { pool }
    }
}

#[async_trait]
impl UserRepository for PostgresUserRepository {
    async fn list(&self, filter: UserFilter) -> Result<PaginatedUsers> {
        // Delegate to separate file
        list_query::list(&self.pool, filter).await
    }

    async fn update(&self, id: Uuid, full_name: Option<String>, is_active: Option<bool>, role_slug: Option<String>, tenant_id: Option<Uuid>, actor_tenant_id: Option<Uuid>) -> Result<bool> {
        let mut tx = self.pool.pool.begin().await?;

        // 1. Update User
        let user_result = sqlx::query(
            "UPDATE auth_users SET 
             full_name = COALESCE($1, full_name), 
             is_active = COALESCE($2, is_active),
             updated_at = NOW()
             WHERE id = $3 AND deleted_at IS NULL
             AND ($4::uuid IS NULL OR EXISTS (
                 SELECT 1 FROM auth_memberships 
                 WHERE user_id = auth_users.id AND tenant_id = $4 AND deleted_at IS NULL
             ))"
        )
        .bind(full_name)
        .bind(is_active)
        .bind(id)
        .bind(actor_tenant_id)
        .execute(&mut *tx)
        .await
        .map_err(|e| anyhow!("Failed to update user: {}", e))?;

        if user_result.rows_affected() == 0 {
            tx.rollback().await?;
            return Ok(false);
        }

        // 2. Update Membership (Role and/or Tenant)
        if role_slug.is_some() || tenant_id.is_some() {
            // Update the existing active membership
            sqlx::query(
                "UPDATE auth_memberships m SET 
                 role_id = COALESCE(r.id, m.role_id),
                 tenant_id = COALESCE($1, m.tenant_id),
                 updated_at = NOW()
                 FROM sys_roles r
                 WHERE m.user_id = $2 AND m.role_id = r.id AND (COALESCE($3, r.slug) = r.slug) AND m.is_active = TRUE AND m.deleted_at IS NULL"
            )
            .bind(tenant_id)
            .bind(id)
            .bind(role_slug)
            .execute(&mut *tx)
            .await
            .map_err(|e| anyhow!("Failed to update user membership: {}", e))?;
        }

        tx.commit().await?;
        Ok(true)
    }
    async fn soft_delete(&self, id: Uuid, actor_tenant_id: Option<Uuid>) -> Result<bool> {
        let mut tx = self.pool.pool.begin().await?;

        // 1. Deactivate User (Only if they belong to the actor's tenant or actor is system owner)
        let result = sqlx::query(
            "UPDATE auth_users SET is_active = FALSE, deleted_at = NOW() 
             WHERE id = $1 AND deleted_at IS NULL
             AND ($2::uuid IS NULL OR EXISTS (
                 SELECT 1 FROM auth_memberships 
                 WHERE user_id = auth_users.id AND tenant_id = $2 AND deleted_at IS NULL
             ))"
        )
            .bind(id)
            .bind(actor_tenant_id)
            .execute(&mut *tx)
            .await
            .map_err(|e| anyhow!("Failed to delete user: {}", e))?;
        
        if result.rows_affected() > 0 {
            // 2. Deactivate Memberships
            sqlx::query("UPDATE auth_memberships SET is_active = FALSE, deleted_at = NOW() WHERE user_id = $1 AND deleted_at IS NULL")
                .bind(id)
                .execute(&mut *tx)
                .await
                .map_err(|e| anyhow!("Failed to deactivate user memberships: {}", e))?;
            
            tx.commit().await?;
            Ok(true)
        } else {
            tx.rollback().await?;
            Ok(false)
        }
    }

    async fn update_password(&self, id: Uuid, hashed_password: String, actor_tenant_id: Option<Uuid>) -> Result<bool> {
        let result = sqlx::query(
            "UPDATE auth_users SET password_hash = $1, updated_at = NOW() 
             WHERE id = $2 AND deleted_at IS NULL
             AND ($3::uuid IS NULL OR EXISTS (
                 SELECT 1 FROM auth_memberships 
                 WHERE user_id = auth_users.id AND tenant_id = $3 AND deleted_at IS NULL
             ))"
        )
        .bind(hashed_password)
        .bind(id)
        .bind(actor_tenant_id)
        .execute(&self.pool.pool)
        .await
        .map_err(|e| anyhow!("Failed to update user password: {}", e))?;
        
        Ok(result.rows_affected() > 0)
    }

    async fn is_superadmin(&self, id: Uuid) -> Result<bool> {
        let is_super: Option<bool> = sqlx::query_scalar(
            "SELECT EXISTS(
                SELECT 1 FROM auth_memberships m 
                JOIN sys_roles r ON m.role_id = r.id 
                WHERE m.user_id = $1 AND r.slug = 'superadmin' AND m.is_active = TRUE AND m.deleted_at IS NULL
            )"
        )
        .bind(id)
        .fetch_one(&self.pool.pool)
        .await
        .map_err(|e| anyhow!("Failed to check superadmin status: {}", e))?;
        Ok(is_super.unwrap_or(false))
    }

    async fn count_superadmins(&self) -> Result<i64> {
        let count: Option<i64> = sqlx::query_scalar(
            "SELECT COUNT(*) FROM auth_memberships m 
             JOIN sys_roles r ON m.role_id = r.id 
             WHERE r.slug = 'superadmin' AND m.is_active = TRUE AND m.deleted_at IS NULL"
        )
        .fetch_one(&self.pool.pool)
        .await
        .map_err(|e| anyhow!("Failed to count superadmins: {}", e))?;
        Ok(count.unwrap_or(0))
    }

    async fn get_user_tenants(&self, id: Uuid) -> Result<Vec<TenantMemberCount>> {
        let tenants = sqlx::query_as::<_, TenantMemberCount>(
            "SELECT t.id, t.name, (
                SELECT COUNT(*)::BIGINT FROM auth_memberships m2 
                WHERE m2.tenant_id = t.id AND m2.is_active = TRUE AND m2.deleted_at IS NULL
            ) as member_count
            FROM auth_memberships m
            JOIN auth_tenants t ON m.tenant_id = t.id
            WHERE m.user_id = $1 AND m.is_active = TRUE AND m.deleted_at IS NULL"
        )
        .bind(id)
        .fetch_all(&self.pool.pool)
        .await
        .map_err(|e| anyhow!("Failed to fetch user tenants: {}", e))?;
        Ok(tenants)
    }
}

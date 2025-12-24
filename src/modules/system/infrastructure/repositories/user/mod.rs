use crate::modules::system::domain::user::{UserEntry, TenantMemberCount, UserRepository, PaginatedUsers, UserFilter};
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

    async fn update(&self, id: Uuid, full_name: Option<String>, is_active: Option<bool>) -> Result<bool> {
        let result = sqlx::query(
            "UPDATE auth_users SET 
             full_name = COALESCE($1, full_name), 
             is_active = COALESCE($2, is_active),
             updated_at = NOW()
             WHERE id = $3 AND deleted_at IS NULL"
        )
        .bind(full_name)
        .bind(is_active)
        .bind(id)
        .execute(&self.pool.pool)
        .await
        .map_err(|e| anyhow!("Failed to update user: {}", e))?;

        Ok(result.rows_affected() > 0)
    }

    async fn soft_delete(&self, id: Uuid) -> Result<bool> {
        let result = sqlx::query("UPDATE auth_users SET deleted_at = NOW() WHERE id = $1 AND deleted_at IS NULL")
            .bind(id)
            .execute(&self.pool.pool)
            .await
            .map_err(|e| anyhow!("Failed to delete user: {}", e))?;
        
        Ok(result.rows_affected() > 0)
    }

    async fn is_superadmin(&self, id: Uuid) -> Result<bool> {
        let is_super = sqlx::query_scalar::<_, bool>(
            "SELECT EXISTS(
                SELECT 1 FROM auth_memberships m 
                JOIN sys_roles r ON m.role_id = r.id 
                WHERE m.user_id = $1 AND r.slug = 'superadmin' AND m.is_active = TRUE
            )"
        )
        .bind(id)
        .fetch_one(&self.pool.pool)
        .await
        .unwrap_or(false);
        Ok(is_super)
    }

    async fn count_superadmins(&self) -> Result<i64> {
        let count = sqlx::query_scalar(
            "SELECT COUNT(*) FROM auth_memberships m 
             JOIN sys_roles r ON m.role_id = r.id 
             WHERE r.slug = 'superadmin' AND m.is_active = TRUE"
        )
        .fetch_one(&self.pool.pool)
        .await
        .unwrap_or(0);
        Ok(count)
    }

    async fn get_user_tenants(&self, id: Uuid) -> Result<Vec<TenantMemberCount>> {
        let tenants = sqlx::query_as::<_, TenantMemberCount>(
            "SELECT t.id, t.name, (
                SELECT COUNT(*) FROM auth_memberships m2 
                WHERE m2.tenant_id = m.tenant_id AND m2.is_active = TRUE
            ) as member_count
            FROM auth_memberships m
            JOIN auth_tenants t ON m.tenant_id = t.id
            WHERE m.user_id = $1 AND m.is_active = TRUE"
        )
        .bind(id)
        .fetch_all(&self.pool.pool)
        .await
        .map_err(|e| anyhow!("Failed to fetch user tenants: {}", e))?;
        Ok(tenants)
    }
}

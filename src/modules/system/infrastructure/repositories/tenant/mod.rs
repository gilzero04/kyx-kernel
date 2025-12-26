use crate::modules::system::domain::tenant::{TenantFilter, PaginatedTenants, TenantRepository};
use crate::core::infrastructure::database::Database;
use async_trait::async_trait;
use anyhow::{Result, anyhow};
use std::sync::Arc;

mod list_query;

pub struct PostgresTenantRepository {
    pool: Arc<Database>,
}

impl PostgresTenantRepository {
    pub fn new(pool: Arc<Database>) -> Self {
        Self { pool }
    }
}

#[async_trait]
impl TenantRepository for PostgresTenantRepository {
    async fn list(&self, filter: TenantFilter) -> Result<PaginatedTenants> {
        list_query::list(&self.pool, filter).await
    }

    async fn update_owner(&self, name: Option<String>, slug: Option<String>) -> Result<()> {
        if let Some(ref s) = slug {
            let exists = sqlx::query_scalar::<_, bool>(
                "SELECT EXISTS(SELECT 1 FROM auth_tenants WHERE slug = $1 AND parent_id IS NOT NULL AND deleted_at IS NULL)"
            )
            .bind(s)
            .fetch_one(&self.pool.pool)
            .await
            .unwrap_or(false);

            if exists {
                return Err(anyhow!("Slug already in use by another organization"));
            }
        }

        sqlx::query(
            "UPDATE auth_tenants SET 
             name = COALESCE($1, name), 
             slug = COALESCE($2, slug),
             updated_at = NOW() 
             WHERE parent_id IS NULL AND deleted_at IS NULL"
        )
        .bind(name)
        .bind(slug)
        .execute(&self.pool.pool)
        .await
        .map_err(|e| anyhow!("Failed to update owner: {}", e))?;
        Ok(())
    }
}

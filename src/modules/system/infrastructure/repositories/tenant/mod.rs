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

    async fn update_owner_name(&self, name: &str) -> Result<()> {
        sqlx::query(
            "UPDATE auth_tenants SET name = $1, updated_at = NOW() WHERE parent_id IS NULL AND deleted_at IS NULL"
        )
        .bind(name)
        .execute(&self.pool.pool)
        .await
        .map_err(|e| anyhow!("Failed to update owner name: {}", e))?;
        Ok(())
    }
}

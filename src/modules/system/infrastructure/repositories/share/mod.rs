use crate::core::infrastructure::database::Database;
use anyhow::{Result, anyhow};
use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use sqlx::FromRow;
use std::sync::Arc;
use uuid::Uuid;

#[derive(Debug, Clone, Serialize, Deserialize, FromRow)]
pub struct ResourceShare {
    pub id: Uuid,
    pub resource_type: String,
    pub resource_id: Uuid,
    pub owner_tenant_id: Uuid,
    pub shared_to_tenant_id: Uuid,
    pub can_reshare: bool,
    pub is_active: bool,
    pub created_by: Option<Uuid>,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}

#[derive(Debug, Deserialize)]
pub struct CreateShareCmd {
    pub resource_type: String,
    pub resource_id: Uuid,
    pub shared_to_tenant_id: Uuid,
    pub can_reshare: Option<bool>,
}

pub struct ShareRepository {
    pool: Arc<Database>,
}

impl ShareRepository {
    pub fn new(pool: Arc<Database>) -> Self {
        Self { pool }
    }

    /// List all shares created by a tenant
    pub async fn list_by_owner(&self, owner_tenant_id: Uuid) -> Result<Vec<ResourceShare>> {
        let shares = sqlx::query_as::<_, ResourceShare>(
            "SELECT * FROM sys_resource_shares WHERE owner_tenant_id = $1 AND is_active = TRUE ORDER BY created_at DESC"
        )
        .bind(owner_tenant_id)
        .fetch_all(&self.pool.pool)
        .await
        .map_err(|e| anyhow!("Failed to list shares: {}", e))?;
        Ok(shares)
    }

    /// List all shares received by a tenant
    pub async fn list_received(&self, tenant_id: Uuid) -> Result<Vec<ResourceShare>> {
        let shares = sqlx::query_as::<_, ResourceShare>(
            "SELECT * FROM sys_resource_shares WHERE shared_to_tenant_id = $1 AND is_active = TRUE ORDER BY created_at DESC"
        )
        .bind(tenant_id)
        .fetch_all(&self.pool.pool)
        .await
        .map_err(|e| anyhow!("Failed to list received shares: {}", e))?;
        Ok(shares)
    }

    /// Create a new share
    pub async fn create(
        &self,
        cmd: CreateShareCmd,
        owner_tenant_id: Uuid,
        created_by: Option<Uuid>,
    ) -> Result<ResourceShare> {
        // Verify ownership of the resource
        let is_owner = self
            .verify_resource_ownership(&cmd.resource_type, cmd.resource_id, owner_tenant_id)
            .await?;
        if !is_owner {
            return Err(anyhow!("Cannot share resource you don't own"));
        }

        // Verify target tenant is within same network (trigger will also check this)
        let can_share = sqlx::query_scalar::<_, bool>(
            "SELECT can_view_tenant($1, $2) OR can_view_tenant($2, $1)",
        )
        .bind(owner_tenant_id)
        .bind(cmd.shared_to_tenant_id)
        .fetch_one(&self.pool.pool)
        .await
        .unwrap_or(false);

        if !can_share {
            return Err(anyhow!("Cannot share resources across different networks"));
        }

        let share = sqlx::query_as::<_, ResourceShare>(
            "INSERT INTO sys_resource_shares (resource_type, resource_id, owner_tenant_id, shared_to_tenant_id, can_reshare, created_by)
             VALUES ($1, $2, $3, $4, $5, $6)
             RETURNING *"
        )
        .bind(&cmd.resource_type)
        .bind(cmd.resource_id)
        .bind(owner_tenant_id)
        .bind(cmd.shared_to_tenant_id)
        .bind(cmd.can_reshare.unwrap_or(false))
        .bind(created_by)
        .fetch_one(&self.pool.pool)
        .await
        .map_err(|e| anyhow!("Failed to create share: {}", e))?;

        Ok(share)
    }

    /// Revoke a share (with usage check)
    pub async fn revoke(
        &self,
        share_id: Uuid,
        actor_tenant_id: Uuid,
    ) -> Result<(bool, Option<i32>)> {
        // Get share details
        let share = sqlx::query_as::<_, ResourceShare>(
            "SELECT * FROM sys_resource_shares WHERE id = $1 AND owner_tenant_id = $2 AND is_active = TRUE"
        )
        .bind(share_id)
        .bind(actor_tenant_id)
        .fetch_optional(&self.pool.pool)
        .await
        .map_err(|e| anyhow!("Failed to find share: {}", e))?;

        let share = match share {
            Some(s) => s,
            None => {
                return Err(anyhow!(
                    "Share not found or you don't have permission to revoke"
                ));
            }
        };

        // Check usage count
        let usage_count: i32 = sqlx::query_scalar("SELECT get_resource_usage_count($1, $2, $3)")
            .bind(&share.resource_type)
            .bind(share.resource_id)
            .bind(share.shared_to_tenant_id)
            .fetch_one(&self.pool.pool)
            .await
            .unwrap_or(0);

        if usage_count > 0 {
            // Return usage count so caller can warn user
            return Ok((false, Some(usage_count)));
        }

        // Safe to revoke - soft delete
        sqlx::query(
            "UPDATE sys_resource_shares SET is_active = FALSE, updated_at = NOW() WHERE id = $1",
        )
        .bind(share_id)
        .execute(&self.pool.pool)
        .await
        .map_err(|e| anyhow!("Failed to revoke share: {}", e))?;

        Ok((true, None))
    }

    /// Force revoke (even if in use)
    pub async fn force_revoke(&self, share_id: Uuid, actor_tenant_id: Uuid) -> Result<bool> {
        let result = sqlx::query(
            "UPDATE sys_resource_shares SET is_active = FALSE, updated_at = NOW() 
             WHERE id = $1 AND owner_tenant_id = $2",
        )
        .bind(share_id)
        .bind(actor_tenant_id)
        .execute(&self.pool.pool)
        .await
        .map_err(|e| anyhow!("Failed to force revoke share: {}", e))?;

        Ok(result.rows_affected() > 0)
    }

    /// Check usage count for a share
    pub async fn get_usage_count(&self, share_id: Uuid) -> Result<i32> {
        let share =
            sqlx::query_as::<_, ResourceShare>("SELECT * FROM sys_resource_shares WHERE id = $1")
                .bind(share_id)
                .fetch_optional(&self.pool.pool)
                .await?;

        let share = match share {
            Some(s) => s,
            None => return Err(anyhow!("Share not found")),
        };

        let count: i32 = sqlx::query_scalar("SELECT get_resource_usage_count($1, $2, $3)")
            .bind(&share.resource_type)
            .bind(share.resource_id)
            .bind(share.shared_to_tenant_id)
            .fetch_one(&self.pool.pool)
            .await
            .unwrap_or(0);

        Ok(count)
    }

    /// Verify resource ownership
    async fn verify_resource_ownership(
        &self,
        resource_type: &str,
        resource_id: Uuid,
        tenant_id: Uuid,
    ) -> Result<bool> {
        let owner_id: Option<Uuid> = match resource_type {
            "role" => {
                sqlx::query_scalar(
                    "SELECT tenant_id FROM sys_roles WHERE id = $1 AND deleted_at IS NULL",
                )
                .bind(resource_id)
                .fetch_optional(&self.pool.pool)
                .await?
            }
            "theme" => {
                sqlx::query_scalar("SELECT tenant_id FROM sys_themes WHERE id = $1")
                    .bind(resource_id)
                    .fetch_optional(&self.pool.pool)
                    .await?
            }
            "page" => {
                sqlx::query_scalar(
                    "SELECT tenant_id FROM sys_pages WHERE id = $1 AND deleted_at IS NULL",
                )
                .bind(resource_id)
                .fetch_optional(&self.pool.pool)
                .await?
            }
            "media" => {
                sqlx::query_scalar(
                    "SELECT tenant_id FROM media_assets WHERE id = $1 AND deleted_at IS NULL",
                )
                .bind(resource_id)
                .fetch_optional(&self.pool.pool)
                .await?
            }
            _ => return Err(anyhow!("Unknown resource type: {}", resource_type)),
        };

        Ok(owner_id == Some(tenant_id))
    }
}

use crate::core::infrastructure::database::Database;
use crate::modules::system::domain::rbac::{
    CreatePermissionCmd, CreateRoleCmd, Permission, RbacRepository, Role, UpdatePermissionCmd,
    UpdateRoleCmd,
};
use anyhow::Result;
use async_trait::async_trait;
use std::sync::Arc;
use uuid::Uuid;

mod permission_query;
mod role_query;

pub struct PostgresRbacRepository {
    pool: Arc<Database>,
}

impl PostgresRbacRepository {
    pub fn new(pool: Arc<Database>) -> Self {
        Self { pool }
    }
}

#[async_trait]
impl RbacRepository for PostgresRbacRepository {
    // === Roles ===
    async fn list_roles(
        &self,
        tenant_id: Option<Uuid>,
        actor_tenant_id: Option<Uuid>,
        show_all: bool,
    ) -> Result<Vec<Role>> {
        role_query::list(&self.pool, tenant_id, actor_tenant_id, show_all).await
    }

    async fn create_role(&self, cmd: CreateRoleCmd, actor_tenant_id: Option<Uuid>) -> Result<Role> {
        role_query::create(&self.pool, cmd, actor_tenant_id).await
    }

    async fn update_role(
        &self,
        id: Uuid,
        cmd: UpdateRoleCmd,
        actor_tenant_id: Option<Uuid>,
    ) -> Result<Option<Role>> {
        role_query::update(&self.pool, id, cmd, actor_tenant_id).await
    }

    async fn delete_role(&self, id: Uuid, actor_tenant_id: Option<Uuid>) -> Result<bool> {
        role_query::delete(&self.pool, id, actor_tenant_id).await
    }

    async fn get_role_permissions(
        &self,
        role_id: Uuid,
        actor_tenant_id: Option<Uuid>,
    ) -> Result<Vec<Permission>> {
        role_query::get_permissions(&self.pool, role_id, actor_tenant_id).await
    }

    async fn update_role_permissions(
        &self,
        role_id: Uuid,
        permission_ids: Vec<Uuid>,
        actor_tenant_id: Option<Uuid>,
    ) -> Result<()> {
        role_query::update_permissions(&self.pool, role_id, permission_ids, actor_tenant_id).await
    }

    // === Permissions ===
    async fn list_permissions(
        &self,
        tenant_id: Option<Uuid>,
        actor_tenant_id: Option<Uuid>,
    ) -> Result<Vec<Permission>> {
        permission_query::list(&self.pool, tenant_id, actor_tenant_id).await
    }

    async fn create_permission(&self, cmd: CreatePermissionCmd) -> Result<Permission> {
        permission_query::create(&self.pool, cmd).await
    }

    async fn update_permission(
        &self,
        id: Uuid,
        cmd: UpdatePermissionCmd,
    ) -> Result<Option<Permission>> {
        permission_query::update(&self.pool, id, cmd).await
    }

    async fn delete_permission(&self, id: Uuid) -> Result<bool> {
        permission_query::delete(&self.pool, id).await
    }
}

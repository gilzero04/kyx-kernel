use crate::modules::system::domain::rbac::{
    RbacRepository, Role, Permission, 
    CreateRoleCmd, UpdateRoleCmd, 
    CreatePermissionCmd, UpdatePermissionCmd
};
use crate::core::infrastructure::database::Database;
use async_trait::async_trait;
use anyhow::Result;
use std::sync::Arc;
use uuid::Uuid;

mod role_query;
mod permission_query;

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
    async fn list_roles(&self) -> Result<Vec<Role>> {
        role_query::list(&self.pool).await
    }

    async fn create_role(&self, cmd: CreateRoleCmd) -> Result<Role> {
        role_query::create(&self.pool, cmd).await
    }

    async fn update_role(&self, id: Uuid, cmd: UpdateRoleCmd) -> Result<Option<Role>> {
        role_query::update(&self.pool, id, cmd).await
    }

    async fn delete_role(&self, id: Uuid) -> Result<bool> {
        role_query::delete(&self.pool, id).await
    }

    // === Permissions ===
    async fn list_permissions(&self) -> Result<Vec<Permission>> {
        permission_query::list(&self.pool).await
    }

    async fn create_permission(&self, cmd: CreatePermissionCmd) -> Result<Permission> {
        permission_query::create(&self.pool, cmd).await
    }

    async fn update_permission(&self, id: Uuid, cmd: UpdatePermissionCmd) -> Result<Option<Permission>> {
        permission_query::update(&self.pool, id, cmd).await
    }

    async fn delete_permission(&self, id: Uuid) -> Result<bool> {
        permission_query::delete(&self.pool, id).await
    }
}

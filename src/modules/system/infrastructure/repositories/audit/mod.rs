use crate::core::infrastructure::database::Database;
use crate::modules::system::domain::audit::{AuditLogFilter, AuditRepository, PaginatedAuditLogs};
use anyhow::Result;
use async_trait::async_trait;
use std::sync::Arc;

mod list_query;

pub struct PostgresAuditRepository {
    pool: Arc<Database>,
}

impl PostgresAuditRepository {
    pub fn new(pool: Arc<Database>) -> Self {
        Self { pool }
    }
}

#[async_trait]
impl AuditRepository for PostgresAuditRepository {
    async fn list(&self, filter: AuditLogFilter) -> Result<PaginatedAuditLogs> {
        list_query::list(&self.pool, filter).await
    }
}

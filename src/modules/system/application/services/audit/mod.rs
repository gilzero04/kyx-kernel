use std::sync::Arc;
use crate::modules::system::domain::audit::{AuditRepository, AuditLogFilter, PaginatedAuditLogs};
use anyhow::Result;

pub struct AuditQueryService {
    repo: Arc<dyn AuditRepository>,
}

impl AuditQueryService {
    pub fn new(repo: Arc<dyn AuditRepository>) -> Self {
        Self { repo }
    }

    pub async fn list_logs(&self, filter: AuditLogFilter) -> Result<PaginatedAuditLogs> {
        self.repo.list(filter).await
    }
}

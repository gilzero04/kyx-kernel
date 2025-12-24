use super::entity::AuditLogEntry;
use anyhow::Result;
use async_trait::async_trait;

pub struct AuditLogFilter {
    pub page: i64,
    pub limit: i64,
    pub sort: Option<String>,
    pub search: Option<String>,
}

use utoipa::ToSchema;

#[derive(Debug, serde::Serialize, ToSchema)]
pub struct PaginationMetadata {
    pub total: i64,
    pub page: i64,
    pub limit: i64,
    pub total_pages: i64,
}

#[derive(Debug, serde::Serialize, ToSchema)]
pub struct PaginatedAuditLogs {
    pub data: Vec<AuditLogEntry>,
    pub pagination: PaginationMetadata,
}

#[async_trait]
pub trait AuditRepository: Send + Sync {
    async fn list(&self, filter: AuditLogFilter) -> Result<PaginatedAuditLogs>;
}

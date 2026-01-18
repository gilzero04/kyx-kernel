use crate::core::AppError;
use crate::core::infrastructure::database::Database;
use serde_json::Value;
use std::sync::Arc;

pub struct AuditService {
    db: Arc<Database>,
}

impl AuditService {
    pub fn new(db: Arc<Database>) -> Self {
        Self { db }
    }

    pub async fn log(
        &self,
        actor: &str,
        action: &str,
        target: Option<&str>,
        status: &str,
        metadata: Option<Value>,
    ) -> Result<(), AppError> {
        sqlx::query(
            r#"
            INSERT INTO audit_logs (actor, action, target, status, metadata)
            VALUES ($1, $2, $3, $4, $5)
            "#,
        )
        .bind(actor)
        .bind(action)
        .bind(target)
        .bind(status)
        .bind(metadata)
        .execute(&self.db.pool)
        .await
        .map_err(|e| AppError {
            code: 500,
            message: format!("Failed to save audit log: {}", e),
        })?;

        // Also log to console for real-time observability
        log::info!(
            "AUDIT: actor={}, action={}, target={:?}, status={}",
            actor,
            action,
            target,
            status
        );

        Ok(())
    }
}

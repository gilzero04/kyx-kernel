use crate::core::infrastructure::database::Database;
use crate::modules::system::domain::audit::{AuditLogEntry, AuditLogFilter, PaginatedAuditLogs};
use anyhow::Result;
use std::sync::Arc;

pub fn get_audit_log_sort_column(col: &str) -> Option<&'static str> {
    match col {
        "timestamp" | "time" => Some("timestamp"),
        "actor" => Some("actor"),
        "action" => Some("action"),
        "target" => Some("target"),
        "status" => Some("status"),
        _ => None,
    }
}

pub fn build_audit_log_order_clause(sort_param: &Option<String>) -> String {
    if let Some(sort) = sort_param {
        let mut order_parts = Vec::new();
        for part in sort.split(',') {
            let parts: Vec<&str> = part.split(':').collect();
            if parts.len() >= 1 {
                let col = parts[0].trim();
                let dir = parts.get(1).map(|d| d.trim()).unwrap_or("asc");
                if let Some(sql_col) = get_audit_log_sort_column(col) {
                    let direction = if dir.eq_ignore_ascii_case("desc") {
                        "DESC"
                    } else {
                        "ASC"
                    };
                    order_parts.push(format!("{} {} NULLS LAST", sql_col, direction));
                }
            }
        }
        if !order_parts.is_empty() {
            return format!("ORDER BY {}", order_parts.join(", "));
        }
    }
    "ORDER BY timestamp DESC".to_string()
}

pub async fn list(pool: &Arc<Database>, filter: AuditLogFilter) -> Result<PaginatedAuditLogs> {
    let page = filter.page.max(1);
    let limit = filter.limit.min(100).max(1);
    let offset = (page - 1) * limit;

    let base_sql = r#"
        SELECT 
            l.id, l.timestamp, l.action, l.status, l.metadata,
            COALESCE(u.email, l.actor) as actor,
            COALESCE(u2.email, t.name, l.target) as target
        FROM audit_logs l
        LEFT JOIN auth_users u ON l.actor = u.id::varchar
        LEFT JOIN auth_users u2 ON l.target = u2.id::varchar
        LEFT JOIN auth_tenants t ON l.target = t.id::varchar
    "#;

    // Simplified count query to avoid unnecessary joins if mostly counting rows,
    // but since search needs joins, we keep them for consistency in filtering.
    let count_base_sql = "SELECT COUNT(*) FROM audit_logs l LEFT JOIN auth_users u ON l.actor = u.id::varchar LEFT JOIN auth_users u2 ON l.target = u2.id::varchar LEFT JOIN auth_tenants t ON l.target = t.id::varchar";

    let order_clause = build_audit_log_order_clause(&filter.sort);

    let (logs_sql, count_sql, search_bind) = if let Some(s) = &filter.search {
        let search_term = format!("%{}%", s);
        let logs = format!(
            "{} WHERE (l.action ILIKE $1 OR l.actor ILIKE $1 OR l.target ILIKE $1 OR u.email ILIKE $1 OR u2.email ILIKE $1 OR t.name ILIKE $1) {} LIMIT $2 OFFSET $3",
            base_sql, order_clause
        );
        let count = format!(
            "{} WHERE (l.action ILIKE $1 OR l.actor ILIKE $1 OR l.target ILIKE $1 OR u.email ILIKE $1 OR u2.email ILIKE $1 OR t.name ILIKE $1)",
            count_base_sql
        );
        (logs, count, Some(search_term))
    } else {
        let logs = format!("{} {} LIMIT $1 OFFSET $2", base_sql, order_clause);
        (logs, count_base_sql.to_string(), None)
    };

    // Execute count query
    let total: i64 = if let Some(ref search) = search_bind {
        sqlx::query_scalar::<_, i64>(&count_sql)
            .bind(search)
            .fetch_one(&pool.pool)
            .await
            .unwrap_or(0)
    } else {
        sqlx::query_scalar::<_, i64>(&count_sql)
            .fetch_one(&pool.pool)
            .await
            .unwrap_or(0)
    };

    // Execute data query
    let entries = if let Some(search) = search_bind {
        sqlx::query_as::<_, AuditLogEntry>(&logs_sql)
            .bind(search)
            .bind(limit)
            .bind(offset)
            .fetch_all(&pool.pool)
            .await?
    } else {
        sqlx::query_as::<_, AuditLogEntry>(&logs_sql)
            .bind(limit)
            .bind(offset)
            .fetch_all(&pool.pool)
            .await?
    };

    let total_pages = (total as f64 / limit as f64).ceil() as i64;

    Ok(PaginatedAuditLogs {
        data: entries,
        pagination: crate::modules::system::domain::audit::PaginationMetadata {
            total,
            page,
            limit,
            total_pages,
        },
    })
}

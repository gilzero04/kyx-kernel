use crate::modules::system::domain::tenant::{TenantEntry, TenantFilter, PaginatedTenants};
use crate::core::infrastructure::database::Database;
use anyhow::{Result, anyhow};
use std::sync::Arc;

pub fn get_tenant_sort_column(col: &str) -> Option<&'static str> {
    match col {
        "name" => Some("t.name"),
        "slug" => Some("t.slug"),
        "is_active" => Some("t.is_active"),
        "created_at" => Some("t.created_at"),
        "member_count" => Some("member_count"),
        _ => None,
    }
}

pub fn build_tenant_order_clause(sort_param: &Option<String>) -> String {
    if let Some(sort) = sort_param {
        let mut order_parts = Vec::new();
        for part in sort.split(',') {
            let parts: Vec<&str> = part.split(':').collect();
            if parts.len() >= 1 {
                let col = parts[0].trim();
                let dir = parts.get(1).map(|d| d.trim()).unwrap_or("asc");
                if let Some(sql_col) = get_tenant_sort_column(col) {
                    let direction = if dir.eq_ignore_ascii_case("desc") { "DESC" } else { "ASC" };
                    order_parts.push(format!("{} {} NULLS LAST", sql_col, direction));
                }
            }
        }
        if !order_parts.is_empty() {
            return format!("ORDER BY {}", order_parts.join(", "));
        }
    }
    "ORDER BY t.created_at DESC".to_string()
}

pub async fn list(pool: &Arc<Database>, filter: TenantFilter) -> Result<PaginatedTenants> {
    let page = filter.page.max(1);
    let limit = filter.limit.min(100).max(1);
    let offset = (page - 1) * limit;

    let base_sql = r#"
        SELECT 
            t.id, t.name, t.slug, t.is_active, t.created_at,
            (SELECT COUNT(*) FROM auth_memberships m WHERE m.tenant_id = t.id) as member_count
        FROM auth_tenants t
        WHERE t.deleted_at IS NULL 
            AND t.parent_id IS NOT NULL
    "#;
    
    let count_base_sql = "SELECT COUNT(*) FROM auth_tenants t WHERE t.deleted_at IS NULL AND t.parent_id IS NOT NULL";

    let order_clause = build_tenant_order_clause(&filter.sort);

    let (data_sql, count_sql, search_bind) = if let Some(s) = &filter.search {
        let search_term = format!("%{}%", s);
        let data = format!(
            "{} AND (t.name ILIKE $1 OR t.slug ILIKE $1) {} LIMIT $2 OFFSET $3",
            base_sql, order_clause
        );
        let count = format!("{} AND (t.name ILIKE $1 OR t.slug ILIKE $1)", count_base_sql);
        (data, count, Some(search_term))
    } else {
        let data = format!("{} {} LIMIT $1 OFFSET $2", base_sql, order_clause);
        (data, count_base_sql.to_string(), None)
    };

    let total: i64 = if let Some(ref search) = search_bind {
        sqlx::query_scalar::<_, i64>(&count_sql).bind(search).fetch_one(&pool.pool).await.unwrap_or(0)
    } else {
        sqlx::query_scalar::<_, i64>(&count_sql).fetch_one(&pool.pool).await.unwrap_or(0)
    };

    let entries = if let Some(search) = search_bind {
        sqlx::query_as::<_, TenantEntry>(&data_sql)
            .bind(search).bind(limit).bind(offset).fetch_all(&pool.pool).await?
    } else {
        sqlx::query_as::<_, TenantEntry>(&data_sql)
            .bind(limit).bind(offset).fetch_all(&pool.pool).await?
    };
    
    let total_pages = (total as f64 / limit as f64).ceil() as i64;

    Ok(PaginatedTenants {
        data: entries,
        pagination: crate::modules::system::domain::tenant::PaginationMetadata {
            total,
            page,
            limit,
            total_pages,
        },
    })
}

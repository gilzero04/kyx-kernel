use crate::modules::system::domain::user::{UserEntry, PaginatedUsers, UserFilter};
use crate::core::infrastructure::database::Database;
use anyhow::Result;
use std::sync::Arc;

fn get_user_sort_column(col: &str) -> Option<&'static str> {
    match col {
        "full_name" | "name" => Some("u.full_name"),
        "email" => Some("u.email"),
        "role" => Some("r.name"),
        "is_active" | "status" => Some("u.is_active"),
        "created_at" | "created" => Some("u.created_at"),
        _ => None,
    }
}

fn build_user_order_clause(sort_param: &Option<String>) -> String {
    if let Some(sort) = sort_param {
        let mut order_parts = Vec::new();
        for part in sort.split(',') {
            let parts: Vec<&str> = part.split(':').collect();
            if parts.len() >= 1 {
                let col = parts[0].trim();
                let dir = parts.get(1).map(|d| d.trim()).unwrap_or("asc");
                if let Some(sql_col) = get_user_sort_column(col) {
                    let direction = if dir.eq_ignore_ascii_case("desc") { "DESC" } else { "ASC" };
                    order_parts.push(format!("{} {} NULLS LAST", sql_col, direction));
                }
            }
        }
        if !order_parts.is_empty() {
            return format!("ORDER BY {}", order_parts.join(", "));
        }
    }
    "ORDER BY u.created_at DESC".to_string()
}

pub async fn list(pool: &Arc<Database>, filter: UserFilter) -> Result<PaginatedUsers> {
    let page = filter.page.max(1);
    let limit = filter.limit.min(100).max(1);
    let offset = (page - 1) * limit;

    let base_sql = r#"
        SELECT 
            u.id, u.email, u.full_name, u.is_active, u.created_at,
            r.name as role,
            r.slug as role_slug,
            t.name as tenant_name,
            t.id as tenant_id
        FROM auth_users u
        LEFT JOIN auth_memberships m ON u.id = m.user_id AND m.is_active = TRUE AND m.deleted_at IS NULL
        LEFT JOIN sys_roles r ON m.role_id = r.id
        LEFT JOIN auth_tenants t ON m.tenant_id = t.id
        WHERE u.deleted_at IS NULL
    "#;

    let count_base_sql = r#"
        SELECT COUNT(DISTINCT u.id)
        FROM auth_users u
        LEFT JOIN auth_memberships m ON u.id = m.user_id AND m.is_active = TRUE AND m.deleted_at IS NULL
        LEFT JOIN sys_roles r ON m.role_id = r.id
        LEFT JOIN auth_tenants t ON m.tenant_id = t.id
        WHERE u.deleted_at IS NULL
    "#;

    let order_clause = build_user_order_clause(&filter.sort);

    let (data_sql, count_sql, search_bind) = if let Some(s) = &filter.search {
        let search_term = format!("%{}%", s);
        let data = format!(
            "{} AND (u.email ILIKE $1 OR u.full_name ILIKE $1 OR r.name ILIKE $1) {} LIMIT $2 OFFSET $3",
            base_sql, order_clause
        );
        let count = format!(
            "{} AND (u.email ILIKE $1 OR u.full_name ILIKE $1 OR r.name ILIKE $1)",
            count_base_sql
        );
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
        sqlx::query_as::<_, UserEntry>(&data_sql)
            .bind(search).bind(limit).bind(offset).fetch_all(&pool.pool).await?
    } else {
        sqlx::query_as::<_, UserEntry>(&data_sql)
            .bind(limit).bind(offset).fetch_all(&pool.pool).await?
    };

    let total_pages = (total as f64 / limit as f64).ceil() as i64;

    Ok(PaginatedUsers {
        data: entries,
        pagination: crate::modules::system::domain::user::PaginationMetadata {
            total,
            page,
            limit,
            total_pages,
        },
    })
}

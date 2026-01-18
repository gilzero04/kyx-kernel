use crate::core::infrastructure::database::Database;
use crate::modules::system::domain::user::{PaginatedUsers, UserEntry, UserFilter};
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
            if !parts.is_empty() {
                let col = parts[0].trim();
                let dir = parts.get(1).map(|d| d.trim()).unwrap_or("asc");
                if let Some(sql_col) = get_user_sort_column(col) {
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
    "ORDER BY u.created_at DESC".to_string()
}

pub async fn list(pool: &Arc<Database>, filter: UserFilter) -> Result<PaginatedUsers> {
    let page = filter.page.max(1);
    let limit = filter.limit.clamp(1, 100);
    let offset = (page - 1) * limit;

    // Build the hierarchy/isolation filter
    // $1 = tree_root, $2 = specific_tenant (optional)
    let (hierarchy_cte, hierarchy_where, p_count) =
        if let Some(_root) = filter.actor_tenant_id.or(filter.tenant_id) {
            (
                r#"WITH RECURSIVE tenant_tree AS (
                SELECT id FROM auth_tenants WHERE id = $1 AND deleted_at IS NULL
                UNION ALL
                SELECT t.id FROM auth_tenants t
                JOIN tenant_tree tt ON t.parent_id = tt.id
                WHERE t.deleted_at IS NULL
            )"#,
                if filter.actor_tenant_id.is_some() && filter.tenant_id.is_some() {
                    "AND t.id = $2 AND t.id IN (SELECT id FROM tenant_tree)"
                } else {
                    "AND t.id IN (SELECT id FROM tenant_tree)"
                },
                if filter.actor_tenant_id.is_some() && filter.tenant_id.is_some() {
                    2
                } else {
                    1
                },
            )
        } else {
            ("", "", 0)
        };

    let base_sql = format!(
        r#"{}
        SELECT 
            u.id, u.email, u.full_name, u.is_active, u.created_at,
            r.name as role,
            r.slug as role_slug,
            t.name as tenant_name,
            t.id as tenant_id,
            t.branding_id as tenant_branding_id,
            NULL::text[] as permissions
        FROM auth_users u
        LEFT JOIN auth_memberships m ON u.id = m.user_id AND m.is_active = TRUE AND m.deleted_at IS NULL
        LEFT JOIN sys_roles r ON m.role_id = r.id
        LEFT JOIN auth_tenants t ON m.tenant_id = t.id
        WHERE u.deleted_at IS NULL {}"#,
        hierarchy_cte, hierarchy_where
    );

    let count_base_sql = format!(
        r#"{}
        SELECT COUNT(DISTINCT u.id)
        FROM auth_users u
        LEFT JOIN auth_memberships m ON u.id = m.user_id AND m.is_active = TRUE AND m.deleted_at IS NULL
        LEFT JOIN sys_roles r ON m.role_id = r.id
        LEFT JOIN auth_tenants t ON m.tenant_id = t.id
        WHERE u.deleted_at IS NULL {}"#,
        hierarchy_cte, hierarchy_where
    );

    let order_clause = build_user_order_clause(&filter.sort);

    let (data_sql, count_sql, search_bind) = if let Some(s) = &filter.search {
        let search_term = format!("%{}%", s);
        let data = format!(
            "{} AND (u.email ILIKE ${} OR u.full_name ILIKE ${} OR r.name ILIKE ${}) {} LIMIT ${} OFFSET ${}",
            base_sql,
            p_count + 1,
            p_count + 1,
            p_count + 1,
            order_clause,
            p_count + 2,
            p_count + 3
        );
        let count = format!(
            "{} AND (u.email ILIKE ${} OR u.full_name ILIKE ${} OR r.name ILIKE ${})",
            count_base_sql,
            p_count + 1,
            p_count + 1,
            p_count + 1
        );
        (data, count, Some(search_term))
    } else {
        let data = format!(
            "{} {} LIMIT ${} OFFSET ${}",
            base_sql,
            order_clause,
            p_count + 1,
            p_count + 2
        );
        (data, count_base_sql.to_string(), None)
    };

    let mut query_total = sqlx::query_scalar::<_, i64>(&count_sql);
    let mut query_data = sqlx::query_as::<_, UserEntry>(&data_sql);

    // Bind parameters in order
    // 1. Hierarchy Root
    if let Some(root) = filter.actor_tenant_id.or(filter.tenant_id) {
        query_total = query_total.bind(root);
        query_data = query_data.bind(root);
    }
    // 2. Specific Tenant (if both aid and tid provided)
    if filter.actor_tenant_id.is_some() && filter.tenant_id.is_some() {
        query_total = query_total.bind(filter.tenant_id.unwrap());
        query_data = query_data.bind(filter.tenant_id.unwrap());
    }

    // Next: Search term
    if let Some(search) = search_bind {
        query_total = query_total.bind(search.clone());
        query_data = query_data.bind(search);
    }

    // Finally: Limit and offset
    query_data = query_data.bind(limit).bind(offset);

    let total = query_total.fetch_one(&pool.pool).await?;
    let entries = query_data.fetch_all(&pool.pool).await?;

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

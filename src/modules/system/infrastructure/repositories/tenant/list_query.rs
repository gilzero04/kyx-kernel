use crate::modules::system::domain::tenant::{TenantEntry, TenantFilter, PaginatedTenants};
use crate::core::infrastructure::database::Database;
use anyhow::Result;
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

    // Build the hierarchy filter if an actor tenant is provided
    let (hierarchy_cte, hierarchy_where) = if let Some(_) = filter.actor_tenant_id {
        (
            r#"WITH RECURSIVE tenant_tree AS (
                -- Anchor: The actor tenant itself
                SELECT id FROM auth_tenants WHERE id = $1 AND deleted_at IS NULL
                UNION
                -- Recursive: Direct and indirect children
                SELECT t.id FROM auth_tenants t
                INNER JOIN tenant_tree tt ON t.parent_id = tt.id
                WHERE t.deleted_at IS NULL
            )"#,
            "AND t.id IN (SELECT id FROM tenant_tree)"
        )
    } else {
        ("", "")
    };

    let base_sql = format!(
        r#"{}
        SELECT 
            t.id, t.parent_id, t.name, t.slug, t.logo_url, t.logo_dark_url, t.favicon_url, t.icon_app_url,
            t.primary_color, t.secondary_color, t.accent_color, t.app_name_override,
            t.contact_email, t.contact_phone, t.website_url, t.social_links, t.address, t.business_type,
            t.config, t.custom_domain, t.allow_child_subdomains, t.use_parent_subdomain,
            t.domain_verified_at, t.verification_token, t.is_active, t.created_at,
            (SELECT COUNT(*) FROM auth_memberships m WHERE m.tenant_id = t.id AND m.deleted_at IS NULL) as member_count
        FROM auth_tenants t
        WHERE t.deleted_at IS NULL {}"#,
        hierarchy_cte, hierarchy_where
    );
    
    let count_base_sql = format!(
        "{} SELECT COUNT(*) FROM auth_tenants t WHERE t.deleted_at IS NULL {}",
        hierarchy_cte, hierarchy_where
    );

    let order_clause = build_tenant_order_clause(&filter.sort);

    // Determine parameter offset based on whether hierarchy_cte is used
    let p_offset = if filter.actor_tenant_id.is_some() { 1 } else { 0 };

    let (data_sql, count_sql, search_bind) = if let Some(s) = &filter.search {
        let search_term = format!("%{}%", s);
        let data = format!(
            "{} AND (t.name ILIKE ${} OR t.slug ILIKE ${}) {} LIMIT ${} OFFSET ${}",
            base_sql, p_offset + 1, p_offset + 1, order_clause, p_offset + 2, p_offset + 3
        );
        let count = format!("{} AND (t.name ILIKE ${} OR t.slug ILIKE ${})", count_base_sql, p_offset + 1, p_offset + 1);
        (data, count, Some(search_term))
    } else {
        let data = format!("{} {} LIMIT ${} OFFSET ${}", base_sql, order_clause, p_offset + 1, p_offset + 2);
        (data, count_base_sql.to_string(), None)
    };

    let mut query_total = sqlx::query_scalar::<_, i64>(&count_sql);
    let mut query_data = sqlx::query_as::<_, TenantEntry>(&data_sql);

    // Bind hierarchy param if exists
    if let Some(tid) = filter.actor_tenant_id {
        query_total = query_total.bind(tid);
        query_data = query_data.bind(tid);
    }

    // Bind search param if exists
    if let Some(search) = search_bind {
        query_total = query_total.bind(search.clone());
        query_data = query_data.bind(search);
    }

    // Bind limit and offset
    query_data = query_data.bind(limit).bind(offset);

    let total = query_total.fetch_one(&pool.pool).await.unwrap_or(0);
    let entries = query_data.fetch_all(&pool.pool).await?;
    
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

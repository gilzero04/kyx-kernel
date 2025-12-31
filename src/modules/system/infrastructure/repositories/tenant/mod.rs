use crate::modules::system::domain::tenant::{TenantFilter, PaginatedTenants, TenantRepository};
use crate::core::infrastructure::database::Database;
use async_trait::async_trait;
use chrono::{DateTime, Utc};
use anyhow::{Result, anyhow};
use std::sync::Arc;
use argon2::{
    password_hash::{
        rand_core::OsRng,
        PasswordHasher, SaltString
    },
    Argon2
};

mod list_query;

pub struct PostgresTenantRepository {
    pool: Arc<Database>,
}

impl PostgresTenantRepository {
    pub fn new(pool: Arc<Database>) -> Self {
        Self { pool }
    }

    async fn resolve_tenant_type(&self, slug: &str) -> Result<sqlx::types::Uuid> {
        sqlx::query_scalar("SELECT id FROM sys_tenant_types WHERE slug = $1")
            .bind(slug)
            .fetch_one(&self.pool.pool)
            .await
            .map_err(|_| anyhow!("Invalid tenant type: {}", slug))
    }
}

#[async_trait]
impl TenantRepository for PostgresTenantRepository {
    async fn list(&self, filter: TenantFilter) -> Result<PaginatedTenants> {
        list_query::list(&self.pool, filter).await
    }

    async fn update_owner(&self, name: Option<String>, slug: Option<String>, logo_url: Option<String>, logo_dark_url: Option<String>, favicon_url: Option<String>, icon_app_url: Option<String>, primary_color: Option<String>, secondary_color: Option<String>, accent_color: Option<String>, app_name_override: Option<String>, contact_email: Option<String>, contact_phone: Option<String>, website_url: Option<String>, social_links: Option<serde_json::Value>, address: Option<String>, business_type: Option<String>, config: Option<serde_json::Value>, custom_domain: Option<String>, allow_child_subdomains: Option<bool>, domain_verified_at: Option<DateTime<Utc>>, verification_token: Option<String>) -> Result<()> {
        if let Some(ref s) = slug {
            let exists = sqlx::query_scalar::<_, bool>(
                "SELECT EXISTS(SELECT 1 FROM auth_tenants WHERE slug = $1 AND parent_id != id AND deleted_at IS NULL)"
            )
            .bind(s)
            .fetch_one(&self.pool.pool)
            .await
            .unwrap_or(false);

            if exists {
                return Err(anyhow!("Slug already in use by another organization"));
            }
        }

        sqlx::query(
            "UPDATE auth_tenants SET 
             name = COALESCE($1, name), 
             slug = COALESCE($2, slug),
             logo_url = COALESCE($3, logo_url),
             logo_dark_url = COALESCE($4, logo_dark_url),
             favicon_url = COALESCE($5, favicon_url),
             icon_app_url = COALESCE($6, icon_app_url),
             primary_color = COALESCE($7, primary_color),
             secondary_color = COALESCE($8, secondary_color),
             accent_color = COALESCE($9, accent_color),
             app_name_override = COALESCE($10, app_name_override),
             contact_email = COALESCE($11, contact_email),
             contact_phone = COALESCE($12, contact_phone),
             website_url = COALESCE($13, website_url),
             social_links = COALESCE($14, social_links),
             address = COALESCE($15, address),
             business_type = COALESCE($16, business_type),
             config = CASE WHEN $17 IS NOT NULL THEN config || $17 ELSE config END,
             custom_domain = COALESCE($18, custom_domain),
             allow_child_subdomains = COALESCE($19, allow_child_subdomains),
             domain_verified_at = COALESCE($20, domain_verified_at),
             verification_token = COALESCE($21, verification_token),
             updated_at = NOW() 
             WHERE parent_id = id AND deleted_at IS NULL"
        )
        .bind(name)
        .bind(slug)
        .bind(logo_url)
        .bind(logo_dark_url)
        .bind(favicon_url)
        .bind(icon_app_url)
        .bind(primary_color)
        .bind(secondary_color)
        .bind(accent_color)
        .bind(app_name_override)
        .bind(contact_email)
        .bind(contact_phone)
        .bind(website_url)
        .bind(social_links)
        .bind(address)
        .bind(business_type)
        .bind(config)
        .bind(custom_domain)
        .bind(allow_child_subdomains)
        .bind(domain_verified_at)
        .bind(verification_token)
        .execute(&self.pool.pool)
        .await
        .map_err(|e| anyhow!("Failed to update owner: {}", e))?;
        Ok(())
    }

    async fn create(&self, name: String, slug: String, type_slug: String, plan: Option<String>, admin_infos: Option<(String, String, String)>, parent_id: Option<sqlx::types::Uuid>, favicon_url: Option<String>, icon_app_url: Option<String>, primary_color: Option<String>, secondary_color: Option<String>, accent_color: Option<String>, app_name_override: Option<String>, contact_email: Option<String>, contact_phone: Option<String>, website_url: Option<String>, social_links: Option<serde_json::Value>, address: Option<String>, business_type: Option<String>) -> Result<crate::modules::system::domain::tenant::entity::TenantEntry> {
        let type_id = self.resolve_tenant_type(&type_slug).await?;
        let config = if let Some(p) = plan {
            serde_json::json!({ "plan": p })
        } else {
            serde_json::json!({})
        };

        let mut tx = self.pool.pool.begin().await?;

        // 1. Create Tenant
        let tenant_id = sqlx::types::Uuid::new_v4();
        let entry = sqlx::query_as::<_, crate::modules::system::domain::tenant::entity::TenantEntry>(
            "INSERT INTO auth_tenants (id, name, slug, tenant_type_id, config, parent_id, is_active, logo_url, logo_dark_url, favicon_url, icon_app_url, primary_color, secondary_color, accent_color, app_name_override, contact_email, contact_phone, website_url, social_links, address, business_type, created_at, updated_at)
             VALUES ($1, $2, $3, $4, $5, COALESCE($6, $1), true, NULL, NULL, $7, $8, $9, $10, $11, $12, $13, $14, $15, $16, $17, $18, NOW(), NOW())
             RETURNING id, parent_id, name, slug, logo_url, logo_dark_url, favicon_url, icon_app_url, primary_color, secondary_color, accent_color, app_name_override, contact_email, contact_phone, website_url, social_links, address, business_type, config, custom_domain, allow_child_subdomains, use_parent_subdomain, domain_verified_at, verification_token, is_active, created_at, NULL as member_count"
        )
        .bind(tenant_id)
        .bind(&name)
        .bind(&slug)
        .bind(type_id)
        .bind(config)
        .bind(parent_id)
        .bind(favicon_url)
        .bind(icon_app_url)
        .bind(primary_color)
        .bind(secondary_color)
        .bind(accent_color)
        .bind(app_name_override)
        .bind(contact_email)
        .bind(contact_phone)
        .bind(website_url)
        .bind(social_links)
        .bind(address)
        .bind(business_type)
        .fetch_one(&mut *tx)
        .await
        .map_err(|e| anyhow!("Failed to create tenant: {}", e))?;

        // 2. Delegate Permissions
        if let Some(pid) = parent_id {
            // Child tenant: Inherit all NON-SYSTEM permissions currently assigned to the parent
            sqlx::query(
                "INSERT INTO sys_tenant_permissions (tenant_id, permission_id)
                 SELECT $1, tp.permission_id FROM sys_tenant_permissions tp
                 JOIN sys_permissions p ON tp.permission_id = p.id
                 WHERE tp.tenant_id = $2 AND p.is_system = FALSE"
            )
            .bind(entry.id)
            .bind(pid)
            .execute(&mut *tx)
            .await
            .map_err(|e| anyhow!("Failed to delegate child permissions: {}", e))?;
        } else {
            // System Owner: Get ALL permissions
            sqlx::query(
                "INSERT INTO sys_tenant_permissions (tenant_id, permission_id)
                 SELECT $1, id FROM sys_permissions"
            )
            .bind(entry.id)
            .execute(&mut *tx)
            .await
            .map_err(|e| anyhow!("Failed to delegate owner permissions: {}", e))?;
        }

        // 3. Create 'superadmin' role for this tenant
        let role_id: sqlx::types::Uuid = sqlx::query_scalar(
            "INSERT INTO sys_roles (id, tenant_id, slug, name, description, is_active, sort_order)
             VALUES (gen_random_uuid(), $1, 'superadmin', 'Tenant Superadmin', 'Full access within this organization', true, 100)
             RETURNING id"
        )
        .bind(entry.id)
        .fetch_one(&mut *tx)
        .await
        .map_err(|e| anyhow!("Failed to create tenant superadmin role: {}", e))?;

        // 4. Map Local SuperAdmin to all delegated permissions
        sqlx::query(
            "INSERT INTO sys_role_permissions (role_id, permission_id)
             SELECT $1, permission_id FROM sys_tenant_permissions WHERE tenant_id = $2"
        )
        .bind(role_id)
        .bind(entry.id)
        .execute(&mut *tx)
        .await
        .map_err(|e| anyhow!("Failed to map permissions to tenant role: {}", e))?;

        // 5. Create/Assign Tenant Admin User
        if let Some((email, password, full_name)) = admin_infos {
            // Check if user exists
            let existing_user_id = sqlx::query_scalar::<_, sqlx::types::Uuid>(
                "SELECT id FROM auth_users WHERE email = $1"
            )
            .bind(&email)
            .fetch_optional(&mut *tx)
            .await?;

            let user_id = if let Some(uid) = existing_user_id {
                uid
            } else {
                // Create new user
                let salt = SaltString::generate(&mut OsRng);
                let argon2 = Argon2::default();
                let password_hash = argon2.hash_password(password.as_bytes(), &salt)
                    .map_err(|e| anyhow!("Password hashing failed: {}", e))?
                    .to_string();

                use crate::core::utils::avatar::generate_default_avatar;
                let default_avatar = generate_default_avatar(&full_name);
                
                sqlx::query_scalar::<_, sqlx::types::Uuid>(
                    "INSERT INTO auth_users (id, email, hashed_password, full_name, avatar_url, is_active, created_at, updated_at)
                     VALUES (gen_random_uuid(), $1, $2, $3, $4, true, NOW(), NOW())
                     RETURNING id"
                )
                .bind(&email)
                .bind(password_hash)
                .bind(&full_name)
                .bind(default_avatar)
                .fetch_one(&mut *tx)
                .await
                .map_err(|e| anyhow!("Failed to create admin user: {}", e))?
            };

            // Assign Membership
            sqlx::query(
                "INSERT INTO auth_memberships (user_id, tenant_id, role, role_id, is_active, created_at, updated_at)
                 VALUES ($1, $2, 'superadmin', $3, true, NOW(), NOW())
                 ON CONFLICT (user_id, tenant_id) DO UPDATE SET role = 'superadmin', role_id = $3, deleted_at = NULL, is_active = TRUE"
            )
            .bind(user_id)
            .bind(entry.id)
            .bind(role_id)
            .execute(&mut *tx)
            .await
            .map_err(|e| anyhow!("Failed to assign admin membership: {}", e))?;
        }

        // 6. CMS: Seed Home Page
        use crate::core::utils::seeding::get_default_home_page_content;
        let home_page_content = get_default_home_page_content(&name);
        
        sqlx::query(
            "INSERT INTO sys_pages (tenant_id, slug, title, content, is_published)
             VALUES ($1, 'home', 'Home', $2, TRUE)"
        )
        .bind(entry.id)
        .bind(home_page_content)
        .execute(&mut *tx)
        .await
        .map_err(|e| anyhow!("Failed to seed home page: {}", e))?;

        tx.commit().await?;
        Ok(entry)
    }

    async fn update_tenant(&self, id: sqlx::types::Uuid, name: Option<String>, slug: Option<String>, is_active: Option<bool>, logo_url: Option<String>, logo_dark_url: Option<String>, favicon_url: Option<String>, icon_app_url: Option<String>, primary_color: Option<String>, secondary_color: Option<String>, accent_color: Option<String>, app_name_override: Option<String>, contact_email: Option<String>, contact_phone: Option<String>, website_url: Option<String>, social_links: Option<serde_json::Value>, address: Option<String>, business_type: Option<String>, config: Option<serde_json::Value>, actor_tenant_id: Option<sqlx::types::Uuid>, custom_domain: Option<String>, allow_child_subdomains: Option<bool>, use_parent_subdomain: Option<bool>, domain_verified_at: Option<DateTime<Utc>>, verification_token: Option<String>) -> Result<crate::modules::system::domain::tenant::entity::TenantEntry> {
        let entry = sqlx::query_as::<_, crate::modules::system::domain::tenant::entity::TenantEntry>(
            "UPDATE auth_tenants SET 
             name = COALESCE($1, name), 
             slug = COALESCE($2, slug),
             is_active = COALESCE($3, is_active),
             logo_url = COALESCE($4, logo_url),
             logo_dark_url = COALESCE($5, logo_dark_url),
             favicon_url = COALESCE($6, favicon_url),
             icon_app_url = COALESCE($7, icon_app_url),
             primary_color = COALESCE($8, primary_color),
             secondary_color = COALESCE($9, secondary_color),
             accent_color = COALESCE($10, accent_color),
             app_name_override = COALESCE($11, app_name_override),
             contact_email = COALESCE($12, contact_email),
             contact_phone = COALESCE($13, contact_phone),
             website_url = COALESCE($14, website_url),
             social_links = COALESCE($15, social_links),
             address = COALESCE($16, address),
             business_type = COALESCE($17, business_type),
             config = CASE WHEN $18 IS NOT NULL THEN config || $18 ELSE config END,
             custom_domain = COALESCE($21, custom_domain),
             allow_child_subdomains = COALESCE($22, allow_child_subdomains),
             use_parent_subdomain = COALESCE($23, use_parent_subdomain),
             domain_verified_at = COALESCE($24, domain_verified_at),
             verification_token = COALESCE($25, verification_token),
             updated_at = NOW() 
             WHERE id = $19 AND deleted_at IS NULL
             AND ($20::uuid IS NULL OR id = $20 OR parent_id = $20)
             RETURNING id, parent_id, name, slug, logo_url, logo_dark_url, favicon_url, icon_app_url, primary_color, secondary_color, accent_color, app_name_override, contact_email, contact_phone, website_url, social_links, address, business_type, config, custom_domain, allow_child_subdomains, use_parent_subdomain, domain_verified_at, verification_token, is_active, created_at, 
             (SELECT COUNT(*) FROM auth_memberships WHERE tenant_id = auth_tenants.id AND deleted_at IS NULL) as member_count"
        )
        .bind(name)
        .bind(slug)
        .bind(is_active)
        .bind(logo_url)
        .bind(logo_dark_url)
        .bind(favicon_url)
        .bind(icon_app_url)
        .bind(primary_color)
        .bind(secondary_color)
        .bind(accent_color)
        .bind(app_name_override)
        .bind(contact_email)
        .bind(contact_phone)
        .bind(website_url)
        .bind(social_links)
        .bind(address)
        .bind(business_type)
        .bind(config)
        .bind(id)
        .bind(actor_tenant_id)
        .bind(custom_domain)
        .bind(allow_child_subdomains)
        .bind(use_parent_subdomain)
        .bind(domain_verified_at)
        .bind(verification_token)
        .fetch_one(&self.pool.pool)
        .await
        .map_err(|e| anyhow!("Failed to update tenant (isolation check failed or not found): {}", e))?;

        Ok(entry)
    }

    async fn delete(&self, id: sqlx::types::Uuid, actor_tenant_id: Option<sqlx::types::Uuid>) -> Result<()> {
        let mut tx = self.pool.pool.begin().await?;

        // 1. Deactivate Tenant (Isolation: must be parent or system owner)
        let row = sqlx::query(
            "UPDATE auth_tenants SET is_active = FALSE, deleted_at = NOW() 
             WHERE id = $1 AND parent_id != id
             AND ($2::uuid IS NULL OR parent_id = $2)"
        )
            .bind(id)
            .bind(actor_tenant_id)
            .execute(&mut *tx)
            .await
            .map_err(|e| anyhow!("Failed to delete tenant: {}", e))?;

        if row.rows_affected() == 0 {
            tx.rollback().await?;
            return Err(anyhow!("Cannot delete owner organization or organization not found"));
        }

        // 2. Deactivate Memberships
        sqlx::query("UPDATE auth_memberships SET is_active = FALSE, deleted_at = NOW() WHERE tenant_id = $1 AND deleted_at IS NULL")
            .bind(id)
            .execute(&mut *tx)
            .await
            .map_err(|e| anyhow!("Failed to deactivate tenant memberships: {}", e))?;

        tx.commit().await?;
        Ok(())
    }

    async fn get_owner_id(&self) -> Result<sqlx::types::Uuid> {
        let id = sqlx::query_scalar::<_, sqlx::types::Uuid>(
            "SELECT id FROM auth_tenants WHERE parent_id = id AND deleted_at IS NULL"
        )
        .fetch_one(&self.pool.pool)
        .await
        .map_err(|e| anyhow!("Failed to fetch owner id: {}", e))?;
        Ok(id)
    }

     async fn get_by_id(&self, id: sqlx::types::Uuid, actor_tenant_id: Option<sqlx::types::Uuid>) -> Result<Option<crate::modules::system::domain::tenant::entity::TenantEntry>> {
        let entry = sqlx::query_as::<_, crate::modules::system::domain::tenant::entity::TenantEntry>(
            "SELECT id, parent_id, name, slug, logo_url, logo_dark_url, favicon_url, icon_app_url, primary_color, secondary_color, accent_color, app_name_override, contact_email, contact_phone, website_url, social_links, address, business_type, config, custom_domain, allow_child_subdomains, use_parent_subdomain, domain_verified_at, verification_token, is_active, created_at, 
             (SELECT COUNT(*) FROM auth_memberships WHERE tenant_id = auth_tenants.id AND deleted_at IS NULL) as member_count
             FROM auth_tenants WHERE id = $1 AND deleted_at IS NULL
             AND ($2::uuid IS NULL OR id = $2 OR parent_id = $2)"
        )
        .bind(id)
        .bind(actor_tenant_id)
        .fetch_optional(&self.pool.pool)
        .await
        .map_err(|e| anyhow!("Failed to fetch tenant: {}", e))?;
        
        Ok(entry)
    }

    async fn get_by_slug(&self, slug: String) -> Result<Option<crate::modules::system::domain::tenant::entity::TenantEntry>> {
        let entry = sqlx::query_as::<_, crate::modules::system::domain::tenant::entity::TenantEntry>(
            "SELECT id, parent_id, name, slug, logo_url, logo_dark_url, favicon_url, icon_app_url, primary_color, secondary_color, accent_color, app_name_override, contact_email, contact_phone, website_url, social_links, address, business_type, config, custom_domain, allow_child_subdomains, use_parent_subdomain, domain_verified_at, verification_token, is_active, created_at, 
             (SELECT COUNT(*) FROM auth_memberships WHERE tenant_id = auth_tenants.id AND deleted_at IS NULL) as member_count
             FROM auth_tenants WHERE slug = $1 AND deleted_at IS NULL AND is_active = TRUE"
        )
        .bind(slug)
        .fetch_optional(&self.pool.pool)
        .await
        .map_err(|e| anyhow!("Failed to fetch tenant by slug: {}", e))?;
        
        Ok(entry)
    }

    #[allow(dead_code)]
    async fn assign_orphaned_records(&self, tenant_id: sqlx::types::Uuid) -> Result<()> {
        let mut tx = self.pool.pool.begin().await?;

        // 1. Update sys_configs (Global settings like branding)
        sqlx::query("UPDATE sys_configs SET tenant_id = $1, scope = COALESCE(scope, 'platform') WHERE tenant_id IS NULL")
            .bind(tenant_id)
            .execute(&mut *tx)
            .await
            .map_err(|e| anyhow!("Failed to assign orphaned sys_configs: {}", e))?;

        // 2. Update sys_themes (Default seeded themes)
        sqlx::query("UPDATE sys_themes SET tenant_id = $1 WHERE tenant_id IS NULL")
            .bind(tenant_id)
            .execute(&mut *tx)
            .await
            .map_err(|e| anyhow!("Failed to assign orphaned sys_themes: {}", e))?;

        // 3. Update sys_pages (If any default pages were seeded without tenant_id)
        sqlx::query("UPDATE sys_pages SET tenant_id = $1 WHERE tenant_id IS NULL")
            .bind(tenant_id)
            .execute(&mut *tx)
            .await
            .map_err(|e| anyhow!("Failed to assign orphaned sys_pages: {}", e))?;

        // 4. Update sys_roles
        sqlx::query("UPDATE sys_roles SET tenant_id = $1 WHERE tenant_id IS NULL")
            .bind(tenant_id)
            .execute(&mut *tx)
            .await
            .map_err(|e| anyhow!("Failed to assign orphaned sys_roles: {}", e))?;

        // 5. Update sys_api_keys
        sqlx::query("UPDATE sys_api_keys SET tenant_id = $1 WHERE tenant_id IS NULL")
            .bind(tenant_id)
            .execute(&mut *tx)
            .await
            .map_err(|e| anyhow!("Failed to assign orphaned sys_api_keys: {}", e))?;

        tx.commit().await?;
        Ok(())
    }
}

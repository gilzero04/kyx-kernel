use std::sync::Arc;
use crate::modules::system::domain::tenant::{TenantRepository, TenantFilter, PaginatedTenants};
// ApiKeyService removed as it is no longer auto-generated during tenant creation
use crate::core::AppError;
use anyhow::Result;
use chrono::{DateTime, Utc};

pub mod domain_verification;

pub struct TenantService {
    repo: Arc<dyn TenantRepository>,
}

impl TenantService {
    pub fn new(repo: Arc<dyn TenantRepository>) -> Self {
        Self { repo }
    }

    pub async fn list_tenants(&self, mut filter: TenantFilter, actor_tenant_id: Option<sqlx::types::Uuid>) -> Result<PaginatedTenants, AppError> {
        filter.actor_tenant_id = actor_tenant_id;
        self.repo.list(filter).await.map_err(|e| AppError {
            code: 500,
            message: e.to_string(),
        })
    }

    pub async fn update_owner(&self, name: Option<String>, slug: Option<String>, logo_url: Option<String>, logo_dark_url: Option<String>, favicon_url: Option<String>, icon_app_url: Option<String>, primary_color: Option<String>, secondary_color: Option<String>, accent_color: Option<String>, app_name_override: Option<String>, contact_email: Option<String>, contact_phone: Option<String>, website_url: Option<String>, social_links: Option<serde_json::Value>, address: Option<String>, business_type: Option<String>, config: Option<serde_json::Value>, custom_domain: Option<String>, allow_child_subdomains: Option<bool>, domain_verified_at: Option<DateTime<Utc>>, verification_token: Option<String>) -> Result<(), AppError> {
        self.repo.update_owner(name, slug, logo_url, logo_dark_url, favicon_url, icon_app_url, primary_color, secondary_color, accent_color, app_name_override, contact_email, contact_phone, website_url, social_links, address, business_type, config, custom_domain, allow_child_subdomains, domain_verified_at, verification_token).await.map_err(|e| AppError {
            code: 500,
            message: e.to_string(),
        })
    }

    pub async fn create_tenant(&self, name: String, slug: String, type_slug: String, plan: Option<String>, admin_email: Option<String>, admin_password: Option<String>, admin_name: Option<String>, mut parent_id: Option<sqlx::types::Uuid>, actor_tenant_id: Option<sqlx::types::Uuid>, favicon_url: Option<String>, icon_app_url: Option<String>, primary_color: Option<String>, secondary_color: Option<String>, accent_color: Option<String>, app_name_override: Option<String>, contact_email: Option<String>, contact_phone: Option<String>, website_url: Option<String>, social_links: Option<serde_json::Value>, address: Option<String>, business_type: Option<String>) -> Result<crate::modules::system::domain::tenant::entity::TenantEntry, AppError> {
        // Enforce hierarchy: Non-system admins can only create children of their own tenant
        if let Some(tid) = actor_tenant_id {
            parent_id = Some(tid);
        }

        if let Some(pid) = parent_id {
            if let Ok(Some(parent)) = self.repo.get_by_id(pid, None).await {
                let parent_plan = parent.config.as_ref()
                    .and_then(|c| c.get("plan"))
                    .and_then(|p| p.as_str())
                    .unwrap_or("multi");

                if parent_plan == "single" && type_slug == "partner" {
                    return Err(AppError {
                        code: 403,
                        message: "Standard CMS plan does not support Partner tenants".to_string(),
                    });
                }
            }
        }
        
        let admin_infos = if let (Some(e), Some(p), Some(n)) = (admin_email, admin_password, admin_name) {
            Some((e, p, n))
        } else {
            None
        };

        let tenant = self.repo.create(name, slug, type_slug, plan, admin_infos, parent_id, favicon_url, icon_app_url, primary_color, secondary_color, accent_color, app_name_override, contact_email, contact_phone, website_url, social_links, address, business_type).await.map_err(|e| AppError {
            code: 500,
            message: e.to_string(),
        })?;

        Ok(tenant)
    }

    pub async fn update_tenant(&self, id: sqlx::types::Uuid, name: Option<String>, slug: Option<String>, is_active: Option<bool>, logo_url: Option<String>, logo_dark_url: Option<String>, favicon_url: Option<String>, icon_app_url: Option<String>, primary_color: Option<String>, secondary_color: Option<String>, accent_color: Option<String>, app_name_override: Option<String>, contact_email: Option<String>, contact_phone: Option<String>, website_url: Option<String>, social_links: Option<serde_json::Value>, address: Option<String>, business_type: Option<String>, config: Option<serde_json::Value>, actor_tenant_id: Option<sqlx::types::Uuid>, custom_domain: Option<String>, allow_child_subdomains: Option<bool>, use_parent_subdomain: Option<bool>, domain_verified_at: Option<DateTime<Utc>>, verification_token: Option<String>) -> Result<crate::modules::system::domain::tenant::entity::TenantEntry, AppError> {
        // Hierarchical subdomain check
        if let Some(true) = use_parent_subdomain {
            if let Ok(Some(tenant)) = self.repo.get_by_id(id, None).await {
                if let Some(pid) = tenant.parent_id {
                    if pid != id { // Not the root tenant
                        if let Ok(Some(parent)) = self.repo.get_by_id(pid, None).await {
                            if !parent.allow_child_subdomains.unwrap_or(false) {
                                return Err(AppError {
                                    code: 403,
                                    message: "Parent tenant does not allow subdomain usage".to_string(),
                                });
                            }
                        }
                    }
                }
            }
        }

        self.repo.update_tenant(id, name, slug, is_active, logo_url, logo_dark_url, favicon_url, icon_app_url, primary_color, secondary_color, accent_color, app_name_override, contact_email, contact_phone, website_url, social_links, address, business_type, config, actor_tenant_id, custom_domain, allow_child_subdomains, use_parent_subdomain, domain_verified_at, verification_token).await.map_err(|e| AppError {
            code: 500,
            message: e.to_string(),
        })
    }

    pub async fn delete_tenant(&self, id: sqlx::types::Uuid, actor_tenant_id: Option<sqlx::types::Uuid>) -> Result<(), AppError> {
        self.repo.delete(id, actor_tenant_id).await.map_err(|e| AppError {
            code: 400,
            message: e.to_string(),
        })
    }

    pub async fn get_owner_id(&self) -> Result<sqlx::types::Uuid, AppError> {
        self.repo.get_owner_id().await.map_err(|e| AppError {
            code: 500,
            message: e.to_string(),
        })
    }

    pub async fn get_tenant_by_id(&self, id: sqlx::types::Uuid, actor_tenant_id: Option<sqlx::types::Uuid>) -> Result<Option<crate::modules::system::domain::tenant::entity::TenantEntry>, AppError> {
        self.repo.get_by_id(id, actor_tenant_id).await.map_err(|e| AppError {
            code: 500,
            message: e.to_string(),
        })
    }

    pub async fn get_tenant_by_slug(&self, slug: String) -> Result<Option<crate::modules::system::domain::tenant::entity::TenantEntry>, AppError> {
        self.repo.get_by_slug(slug).await.map_err(|e| AppError {
            code: 500,
            message: e.to_string(),
        })
    }
}

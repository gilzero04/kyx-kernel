use std::sync::Arc;
use trust_dns_resolver::TokioAsyncResolver;
use trust_dns_resolver::config::{ResolverConfig, ResolverOpts};
use trust_dns_resolver::proto::rr::RecordType;
use chrono::{Utc, DateTime};
use anyhow::Result;
use crate::modules::system::domain::tenant::TenantRepository;
use crate::core::AppError;

pub struct DomainVerificationService {
    repo: Arc<dyn TenantRepository>,
}

impl DomainVerificationService {
    pub fn new(repo: Arc<dyn TenantRepository>) -> Self {
        Self { repo }
    }

    /// Verifies custom domain ownership and connectivity
    pub async fn verify_domain(&self, tenant_id: uuid::Uuid, custom_domain: &str) -> Result<DateTime<Utc>, AppError> {
        let tenant = self.repo.get_by_id(tenant_id, None).await.map_err(|e| AppError {
            code: 500,
            message: e.to_string(),
        })?.ok_or_else(|| AppError {
            code: 404,
            message: "Tenant not found".to_string(),
        })?;

        let token = tenant.verification_token.ok_or_else(|| AppError {
            code: 500,
            message: "Missing verification token".to_string(),
        })?;

        // 1. Initialize Resolver
        let resolver = TokioAsyncResolver::tokio(
            ResolverConfig::default(),
            ResolverOpts::default(),
        );

        // 2. Check TXT Record for Ownership (Cloudflare-style)
        // Expected record: kyx-verification=[token]
        let txt_query = format!("_kyx-verify.{}", custom_domain);
        let txt_lookup = resolver.lookup(txt_query, RecordType::TXT).await;
        
        let owned = match txt_lookup {
            Ok(lookup) => {
                lookup.record_iter().any(|record| {
                    if let Some(txt) = record.data().and_then(|data| data.as_txt()) {
                        txt.iter().any(|data| {
                            let s = String::from_utf8_lossy(data);
                            s == format!("kyx-verification={}", token)
                        })
                    } else {
                        false
                    }
                })
            }
            Err(_) => false,
        };

        if !owned {
            return Err(AppError {
                code: 400,
                message: "DNS Ownership verification failed (TXT record not found or incorrect)".to_string(),
            });
        }

        // 3. Check CNAME for Connectivity
        let cname_lookup = resolver.lookup(custom_domain, RecordType::CNAME).await;
        if cname_lookup.is_err() {
            // Fallback: Check if it has an A record at least
            let a_lookup = resolver.lookup(custom_domain, RecordType::A).await;
            if a_lookup.is_err() {
                 return Err(AppError {
                    code: 400,
                    message: "DNS Connectivity failed (No CNAME or A record found)".to_string(),
                });
            }
        }

        let now = Utc::now();
        self.repo.update_tenant(
            tenant_id,
            None, // name
            None, // slug
            None, // is_active
            None, // branding_id
            None, // contact_email
            None, // contact_phone
            None, // website_url
            None, // social_links
            None, // address
            None, // business_type
            None, // config
            None, // actor_tenant_id
            None, // custom_domain
            None, // allow_child_subdomains
            None, // use_parent_subdomain
            Some(now), // domain_verified_at
            None // verification_token
        ).await.map_err(|e| AppError {
            code: 500,
            message: e.to_string(),
        })?;

        Ok(now)
    }
}

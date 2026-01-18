// ═══════════════════════════════════════════════════════════════════════════════
// Context Guard — Cross-Context Access Prevention
// ═══════════════════════════════════════════════════════════════════════════════
// Prevents unauthorized cross-context access:
// - Console events should not modify Workspace state
// - Workspace events should not modify Console state
// - Tenant A should not access Tenant B data
// ═══════════════════════════════════════════════════════════════════════════════

use serde::{Deserialize, Serialize};
use uuid::Uuid;

/// Application context
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum AppContext {
    Console,
    Workspace,
    Storefront,
    Internal,
}

impl AppContext {
    pub fn from_path(path: &str) -> Self {
        if path.starts_with("/console") {
            AppContext::Console
        } else if path.starts_with("/workspace") {
            AppContext::Workspace
        } else if path.starts_with("/s/") {
            AppContext::Storefront
        } else if path.starts_with("/internal") || path.starts_with("/api/v1/admin") {
            // Admin routes are considered Console context for permission purposes
            AppContext::Console
        } else {
            AppContext::Internal
        }
    }

    pub fn as_str(&self) -> &'static str {
        match self {
            AppContext::Console => "console",
            AppContext::Workspace => "workspace",
            AppContext::Storefront => "storefront",
            AppContext::Internal => "internal",
        }
    }
}

/// Context guard for validating cross-context access
pub struct ContextGuard;

/// Result of context validation
#[derive(Debug)]
pub enum ContextValidation {
    Allowed,
    Denied(String),
}

impl ContextGuard {
    /// Validate that a request from a given context can access the target resource
    ///
    /// Rules:
    /// - Console can access Console resources
    /// - Console can access Workspace resources (superadmin)
    /// - Workspace can access Workspace resources (same tenant or child)
    /// - Workspace cannot access Console resources
    /// - Storefront can only access public resources
    pub fn validate(
        request_context: AppContext,
        target_context: AppContext,
        request_tenant_id: Option<Uuid>,
        target_tenant_id: Option<Uuid>,
    ) -> ContextValidation {
        use AppContext::*;

        match (request_context, target_context) {
            // Console can access everything
            (Console, _) => ContextValidation::Allowed,

            // Internal can access everything
            (Internal, _) => ContextValidation::Allowed,

            // Workspace can access Workspace (with tenant check)
            (Workspace, Workspace) => {
                if let (Some(req_tid), Some(target_tid)) = (request_tenant_id, target_tenant_id) {
                    if req_tid == target_tid {
                        ContextValidation::Allowed
                    } else {
                        // Parent-child check: Requester can access if target is descendant
                        // NOTE: Full hierarchy check requires DB query via get_ancestor_chain()
                        // For sync validation, we deny cross-tenant access here.
                        // Caller should pre-validate parent-child relationship via TenantService
                        ContextValidation::Denied(format!(
                            "Tenant {} cannot access tenant {} resources. Pre-validate hierarchy via TenantService if parent-child access is needed.",
                            req_tid, target_tid
                        ))
                    }
                } else {
                    ContextValidation::Allowed
                }
            }

            // Workspace cannot access Console
            (Workspace, Console) => ContextValidation::Denied(
                "Workspace context cannot access Console resources".to_string(),
            ),

            // Storefront can only access Storefront
            (Storefront, Storefront) => ContextValidation::Allowed,

            // Storefront cannot access admin contexts
            (Storefront, Console) | (Storefront, Workspace) => {
                ContextValidation::Denied("Storefront cannot access admin resources".to_string())
            }

            // Allow Workspace and Storefront same-tenant access
            (Workspace, Storefront) => ContextValidation::Allowed,
            (Storefront, Internal) => ContextValidation::Allowed,
            (Workspace, Internal) => ContextValidation::Allowed,
        }
    }

    /// Check if access is allowed (convenience method)
    pub fn is_allowed(
        request_context: AppContext,
        target_context: AppContext,
        request_tenant_id: Option<Uuid>,
        target_tenant_id: Option<Uuid>,
    ) -> bool {
        matches!(
            Self::validate(
                request_context,
                target_context,
                request_tenant_id,
                target_tenant_id
            ),
            ContextValidation::Allowed
        )
    }
}

// ═══════════════════════════════════════════════════════════════════════════════
// Tests
// ═══════════════════════════════════════════════════════════════════════════════

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_context_from_path() {
        assert_eq!(
            AppContext::from_path("/console/dashboard"),
            AppContext::Console
        );
        assert_eq!(
            AppContext::from_path("/workspace/users"),
            AppContext::Workspace
        );
        assert_eq!(
            AppContext::from_path("/s/my-store/products"),
            AppContext::Storefront
        );
        assert_eq!(
            AppContext::from_path("/api/v1/admin/users"),
            AppContext::Console
        );
    }

    #[test]
    fn test_console_can_access_all() {
        assert!(ContextGuard::is_allowed(
            AppContext::Console,
            AppContext::Workspace,
            None,
            None,
        ));
    }

    #[test]
    fn test_workspace_cannot_access_console() {
        assert!(!ContextGuard::is_allowed(
            AppContext::Workspace,
            AppContext::Console,
            Some(Uuid::new_v4()),
            None,
        ));
    }

    #[test]
    fn test_workspace_same_tenant() {
        let tenant_id = Uuid::new_v4();
        assert!(ContextGuard::is_allowed(
            AppContext::Workspace,
            AppContext::Workspace,
            Some(tenant_id),
            Some(tenant_id),
        ));
    }

    #[test]
    fn test_workspace_different_tenant() {
        assert!(!ContextGuard::is_allowed(
            AppContext::Workspace,
            AppContext::Workspace,
            Some(Uuid::new_v4()),
            Some(Uuid::new_v4()),
        ));
    }

    #[test]
    fn test_storefront_cannot_access_admin() {
        assert!(!ContextGuard::is_allowed(
            AppContext::Storefront,
            AppContext::Console,
            None,
            None,
        ));
    }
}

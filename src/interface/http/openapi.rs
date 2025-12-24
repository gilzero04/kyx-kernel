use utoipa::{OpenApi, Modify, openapi::security::{SecurityScheme, HttpBuilder, HttpAuthScheme}};
use crate::modules::system::interface::http::handlers::{config, system, rbac, tenant, user, audit, api_key, cors, i18n};
use crate::modules::auth::interface::http::handlers::auth;
use crate::modules::media::interface::http::handlers::media;

// Schemas
use crate::modules::system::interface::http::dto::config::ConfigUpdate;
use crate::modules::system::interface::http::dto::rbac::{CreateRoleRequest, UpdateRoleRequest, CreatePermissionRequest, UpdatePermissionRequest};
use crate::modules::system::interface::http::dto::tenant::{TenantsQuery, UpdateOwnerRequest};
use crate::modules::system::interface::http::dto::user::{UsersQuery, UpdateUserRequest};
use crate::modules::system::interface::http::dto::audit::LogsQuery;
use crate::modules::system::interface::http::dto::api_key::CreateApiKeyRequest;
use crate::modules::system::interface::http::dto::cors::AddCorsRequest;
use crate::modules::system::interface::http::dto::i18n::{TranslationsResponse, CreateI18nKeyRequest, UpdateTranslationRequest, CreateLocaleRequest};

use crate::modules::auth::interface::http::dto::auth::{AuthResponse, UserInfo, SetupRequest, CreateUserRequest, RefreshRequest};
use crate::modules::auth::domain::login::UserCredentials;

// Entities
use crate::modules::system::domain::rbac::{Role, Permission};
use crate::modules::system::domain::tenant::{TenantEntry, PaginatedTenants};
use crate::modules::system::domain::user::{UserEntry, PaginatedUsers};
use crate::modules::system::domain::api_key::ApiKey;
use crate::modules::system::domain::cors::CorsOrigin;
use crate::modules::system::domain::audit::AuditLogEntry;

#[derive(OpenApi)]
#[openapi(
    paths(
        // System / Config
        config::get_config,
        config::update_config,
        system::get_system_info,
        system::get_system_settings,
        system::get_system_status,
        
        // RBAC
        rbac::list_roles,
        rbac::create_role,
        rbac::update_role,
        rbac::delete_role,
        rbac::list_permissions,
        rbac::create_permission,
        rbac::update_permission,
        rbac::delete_permission,

        // Tenants
        tenant::list_tenants,
        tenant::update_owner,

        // Users
        user::list_users,
        user::update_user,
        user::delete_user,

        // Audit
        audit::list_audit_logs,

        // API Keys
        api_key::create_api_key,

        // CORS
        cors::list_cors_origins,
        cors::add_cors_origin,
        cors::delete_cors_origin,

        // I18n
        i18n::list_locales,
        i18n::get_translations,
        i18n::create_key,
        i18n::update_translation,
        i18n::create_locale,

        // Auth
        auth::login,
        auth::refresh_session,
        auth::logout,
        auth::get_setup_status,
        auth::verify_engine_key,
        auth::initialize_system,
        auth::create_user,

        // Media
        media::upload_file,
        media::serve_file,
    ),
    components(
        schemas(
            ConfigUpdate, 
            AuthResponse, 
            UserInfo, 
            SetupRequest, 
            CreateUserRequest, 
            RefreshRequest, 
            UserCredentials,
            CreateRoleRequest,
            UpdateRoleRequest,
            CreatePermissionRequest,
            UpdatePermissionRequest,
            TenantsQuery,
            UpdateOwnerRequest,
            UsersQuery,
            UpdateUserRequest,
            LogsQuery,
            CreateApiKeyRequest,
            AddCorsRequest,
            TranslationsResponse,
            CreateI18nKeyRequest,
            UpdateTranslationRequest,
            CreateLocaleRequest,
            Role,
            Permission,
            TenantEntry,
            PaginatedTenants,
            UserEntry,
            PaginatedUsers,
            ApiKey,
            CorsOrigin,
            AuditLogEntry
        )
    ),
    tags(
        (name = "status", description = "Core system status, health, and branding information"),
        (name = "settings", description = "System-wide configuration and administrative settings"),
        (name = "auth", description = "Authentication and user session management"),
        (name = "rbac", description = "Role-Based Access Control (Permissions & Roles)"),
        (name = "users", description = "User administration and management"),
        (name = "tenants", description = "Multi-tenancy and organization management"),
        (name = "audit", description = "Security audit logs and activity tracking"),
        (name = "api-keys", description = "API Key management for external integrations"),
        (name = "cors", description = "Cross-Origin Resource Sharing (CORS) configuration"),
        (name = "i18n", description = "Internationalization and translation management"),
        (name = "media", description = "Media storage and file serving"),
    ),
    info(
        title = "Kyx Kernel API",
        version = "1.0.0",
        description = "Enterprise Plugin Platform Orchestrator"
    ),
    modifiers(&SecurityAddon)
)]
pub struct ApiDoc;

struct SecurityAddon;

impl Modify for SecurityAddon {
    fn modify(&self, openapi: &mut utoipa::openapi::OpenApi) {
        let components = openapi.components.as_mut().unwrap();
        components.add_security_scheme(
            "bearer_auth",
            SecurityScheme::Http(
                HttpBuilder::new()
                    .scheme(HttpAuthScheme::Bearer)
                    .bearer_format("JWT")
                    .build(),
            ),
        )
    }
}

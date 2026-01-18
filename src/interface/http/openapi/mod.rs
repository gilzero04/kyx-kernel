use crate::modules::auth::interface::http::handlers::{auth, user_preferences};
use crate::modules::media::interface::http::handlers::media;
use crate::modules::signal::interface::http::handlers as signal;
use crate::modules::system::interface::http::handlers::{
    api_key, audit, config, cors, i18n, plugin, rbac, system, tenant, theme, user,
};
use utoipa::{
    Modify, OpenApi,
    openapi::security::{HttpAuthScheme, HttpBuilder, SecurityScheme},
};

// Schemas
use crate::modules::system::interface::http::dto::api_key::CreateApiKeyRequest;
use crate::modules::system::interface::http::dto::audit::LogsQuery;
use crate::modules::system::interface::http::dto::config::ConfigUpdate;
use crate::modules::system::interface::http::dto::cors::AddCorsRequest;
use crate::modules::system::interface::http::dto::i18n::{
    CreateI18nKeyRequest, CreateLocaleRequest, TranslationsResponse, UpdateTranslationRequest,
};
use crate::modules::system::interface::http::dto::rbac::{
    CreatePermissionRequest, CreateRoleRequest, UpdatePermissionRequest, UpdateRoleRequest,
};
use crate::modules::system::interface::http::dto::tenant::{TenantsQuery, UpdateOwnerRequest};
use crate::modules::system::interface::http::dto::user::{UpdateUserRequest, UsersQuery};
// Note: CreateFolderRequest and AssetQuery available via media::dto when needed

use crate::modules::auth::domain::login::UserCredentials;
use crate::modules::auth::interface::http::dto::auth::{
    AdminSessionInfo, AuthResponse, CreateUserRequest, RefreshRequest, SessionInfo, SetupRequest,
    SignupRequest, UserImageInfo, UserInfo,
};
use crate::modules::system::domain::user::repository::PaginationMetadata;

// Entities
use crate::modules::system::domain::api_key::ApiKey;
use crate::modules::system::domain::audit::AuditLogEntry;
use crate::modules::system::domain::cors::CorsOrigin;
use crate::modules::system::domain::rbac::{Permission, Role};
use crate::modules::system::domain::tenant::{PaginatedTenants, TenantEntry};
use crate::modules::system::domain::user::{PaginatedUsers, UserEntry};

// Plugin Schemas
use crate::modules::system::domain::plugin::entity::{Author, Capability, Manifest, RuntimeType};
use crate::modules::system::domain::plugin::registry::{RiskLevel, SecuritySummary};
use crate::modules::system::interface::http::handlers::plugin::{
    AnalyzePluginRequest, ErrorResponse, InstallPluginRequest, InstallWithApprovalRequest,
    PluginListResponse, PluginResponse, SecurityWarningsResponse, SuccessResponse, TenantQuery,
    UpdatePluginConfigRequest,
};

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
        rbac::get_role_permissions,
        rbac::update_role_permissions,
        rbac::list_permissions,
        rbac::create_permission,
        rbac::update_permission,
        rbac::delete_permission,

        // Tenants
        tenant::list_tenants,
        tenant::get_tenant,
        tenant::create_tenant,
        tenant::update_tenant,
        tenant::delete_tenant,
        tenant::update_owner,
        tenant::verify_domain,
        tenant::get_tenant_by_slug,
        tenant::get_current_tenant,

        // Users
        user::list_users,
        user::update_user,
        user::delete_user,
        user::reset_password,

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
        auth::check_slug_availability,
        auth::initialize_system,
        auth::signup,
        auth::create_user,

        // Sessions
        auth::list_sessions,
        auth::revoke_session,
        auth::admin_list_sessions,
        auth::admin_revoke_session_handler,

        // Media
        media::upload_file,
        media::serve_file,
        media::create_folder,
        media::list_assets,
        media::delete_asset,

        // User Preferences
        user_preferences::get_my_preferences,
        user_preferences::update_my_preferences,

        // Themes
        theme::list_themes,
        theme::import_theme,
        theme::set_active_theme,
        theme::delete_theme,
        theme::set_sharing,

        // Plugins
        plugin::list_plugins,
        plugin::get_plugin,
        plugin::install_plugin,
        plugin::enable_plugin,
        plugin::disable_plugin,
        plugin::uninstall_plugin,
        plugin::update_plugin_config,
        plugin::analyze_plugin_security,
        plugin::install_plugin_with_approval,
        plugin::get_plugin_security,

        // Signal
        signal::generate_signal_token,
        signal::signal_health,
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
            AuditLogEntry,
            // Plugin schemas
            PluginResponse,
            PluginListResponse,
            SuccessResponse,
            ErrorResponse,
            InstallPluginRequest,
            UpdatePluginConfigRequest,
            TenantQuery,
            AnalyzePluginRequest,
            InstallWithApprovalRequest,
            SecurityWarningsResponse,
            SecuritySummary,
            RiskLevel,
            // Plugin entity schemas
            Manifest,
            Author,
            Capability,
            RuntimeType,
            // Pagination and media schemas
            PaginationMetadata,
            UserImageInfo,
            // Session schemas
            SignupRequest,
            SessionInfo,
            AdminSessionInfo
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
        (name = "plugins", description = "Plugin management and security"),
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

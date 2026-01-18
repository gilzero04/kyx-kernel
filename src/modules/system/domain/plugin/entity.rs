// ════════════════════════════════════════════════════════════════════════════
// Plugin Entity - Extended Domain Model
// Compatible with: kyx-engine, pixco-customer-app, external-projects
// ════════════════════════════════════════════════════════════════════════════

use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use std::collections::HashSet;
use uuid::Uuid;

// ════════════════════════════════════════════════════════════════════════════
// Plugin Manifest (Full Compatibility with pixco manifest.json)
// ════════════════════════════════════════════════════════════════════════════

/// Plugin manifest describing capabilities and metadata
/// Compatible with pixco/kyx-engine external plugin format
#[derive(Debug, Clone, Serialize, Deserialize, utoipa::ToSchema)]
#[serde(rename_all = "camelCase")]
pub struct Manifest {
    // ─── Core Identity ───────────────────────────────────────────────────────
    pub id: String,
    pub name: String,
    pub version: String,
    #[serde(default)]
    pub description: Option<String>,

    // ─── Entry Point ─────────────────────────────────────────────────────────
    #[serde(default)]
    pub entry: Option<String>, // "./index.js" for frontend, path to WASM for backend

    // ─── Author Information ──────────────────────────────────────────────────
    #[serde(default)]
    pub author: Option<Author>,

    // ─── Display & Branding ──────────────────────────────────────────────────
    #[serde(default)]
    pub icon: Option<String>, // Emoji or URL
    #[serde(default)]
    pub banner: Option<String>, // Banner image URL
    #[serde(default)]
    pub category: Option<String>, // "Entertainment", "Developer Tools", etc.
    #[serde(default)]
    pub tags: Vec<String>,

    // ─── Security & Permissions ──────────────────────────────────────────────
    #[serde(default)]
    pub permissions: Vec<String>, // ["camera", "microphone"]
    #[serde(default)]
    pub capabilities: HashSet<Capability>, // Kernel capabilities (granular)
    #[serde(default)]
    pub data_access: Vec<String>, // ["user_profile", "orders"]
    #[serde(default)]
    pub network_access: Vec<String>, // ["api.example.com"]

    // ─── Runtime Configuration ───────────────────────────────────────────────
    #[serde(default = "default_runtime")]
    pub runtime: RuntimeType,

    // ─── Compatibility ───────────────────────────────────────────────────────
    #[serde(default)]
    pub min_app_version: Option<String>, // "2.0.0"
    #[serde(default)]
    pub max_app_version: Option<String>, // Optional upper bound

    // ─── Metadata ────────────────────────────────────────────────────────────
    #[serde(default)]
    pub release_date: Option<String>, // "2024-12-15"
    #[serde(default)]
    pub changelog: Option<String>,
    #[serde(default)]
    pub status: Option<String>, // "stable", "beta", "deprecated"

    // ─── Trust Indicators ────────────────────────────────────────────────────
    #[serde(default)]
    pub verified: bool, // Platform verified
    #[serde(default)]
    pub official: bool, // Official KYX plugin
    #[serde(default)]
    pub featured: bool, // Featured in marketplace
    #[serde(default)]
    pub is_core_plugin: bool, // Core platform functionality

    // ─── Links & Documentation ───────────────────────────────────────────────
    #[serde(default)]
    pub homepage: Option<String>,
    #[serde(default)]
    pub documentation: Option<String>,
    #[serde(default)]
    pub repository: Option<String>,
    #[serde(default)]
    pub support: Option<String>,
    #[serde(default)]
    pub license: Option<String>,
    #[serde(default)]
    pub privacy_policy: Option<String>,

    // ─── Sharing & Inheritance ───────────────────────────────────────────────
    #[serde(default)]
    pub visibility: PluginVisibility, // private, shared, global

    // ─── UI Extensions ───────────────────────────────────────────────────────
    #[serde(default)]
    pub ui: Option<UIExtensions>,
}

/// Helper to default visibility
fn default_visibility() -> PluginVisibility {
    PluginVisibility::Private
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, utoipa::ToSchema)]
#[serde(rename_all = "camelCase")]
#[derive(Default)]
pub enum PluginVisibility {
    #[default]
    Private,
    Shared,
    Global,
}


/// UI Definitions for the plugin
#[derive(Debug, Clone, Serialize, Deserialize, utoipa::ToSchema, PartialEq)]
#[serde(rename_all = "camelCase")]
pub struct UIExtensions {
    #[serde(default)]
    pub menus: Vec<MenuExtension>,
}

/// A single menu item definition
#[derive(Debug, Clone, Serialize, Deserialize, utoipa::ToSchema, PartialEq)]
#[serde(rename_all = "camelCase")]
pub struct MenuExtension {
    /// Unique ID for the menu item (e.g. "blog-dashboard")
    pub id: String,

    /// i18n Translation Key
    /// MUST follow pattern: `plugin.{plugin_id}.{suffix}`
    /// e.g. "plugin.blog.menu.dashboard"
    pub label: String,

    /// Target path or URL
    pub path: String,

    /// Icon definition
    /// - Standard: Lucide/Material string (e.g. "users")
    /// - Custom: SVG string (starts with "<svg>")
    /// - Remote: URL (starts with "http")
    pub icon: Option<String>,

    /// Target Zone ID (e.g. "sidebar.main", "mobile.bottom_nav")
    pub parent_id: Option<String>,

    /// Sort order
    #[serde(default)]
    pub order: i32,

    /// Required RBAC permissions to view this item
    #[serde(default)]
    pub permissions: Vec<String>,

    /// Open in new tab?
    #[serde(default)]
    pub external: bool,
}

fn default_runtime() -> RuntimeType {
    RuntimeType::Frontend
}

/// Author information
#[derive(Debug, Clone, Serialize, Deserialize, Default, utoipa::ToSchema)]
pub struct Author {
    pub name: String,
    #[serde(default)]
    pub email: Option<String>,
    #[serde(default)]
    pub website: Option<String>,
}

// ════════════════════════════════════════════════════════════════════════════
// Plugin Capabilities (Kernel-level Permissions)
// ════════════════════════════════════════════════════════════════════════════

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq, Hash, utoipa::ToSchema)]
#[serde(rename_all = "snake_case")]
pub enum Capability {
    // ─── Storage ─────────────────────────────────────────────────────────────
    StorageRead,
    StorageWrite,

    // ─── Events ──────────────────────────────────────────────────────────────
    EventEmit,
    EventSubscribe,

    // ─── HTTP (outbound only) ────────────────────────────────────────────────
    HttpRequest,

    // ─── UI Extensions ───────────────────────────────────────────────────────
    UiRegisterPage,
    UiRegisterWidget,
    UiRegisterSidebar,
    UiRegisterAdminPage,

    // ─── Logging ─────────────────────────────────────────────────────────────
    LogInfo,
    LogWarn,
    LogError,

    // ─── Device Access ───────────────────────────────────────────────────────
    Camera,
    Microphone,
    Geolocation,
    Notifications,

    // ─── Data Access ─────────────────────────────────────────────────────────
    UserProfileRead,
    UserProfileWrite,
    TenantDataRead,
    TenantDataWrite,

    // ─── Legacy (deprecated, use granular) ───────────────────────────────────
    #[serde(rename = "api")]
    Api,
    #[serde(rename = "ui")]
    Ui,
    #[serde(rename = "storage")]
    Storage,
    #[serde(rename = "event")]
    Event,

    // ─── Financial (REQUIRES EXPLICIT APPROVAL - Rule 4) ─────────────────────
    #[serde(rename = "financial_read")]
    FinancialRead,
    #[serde(rename = "financial_write")]
    FinancialWrite, // NEVER auto-granted
}

impl Capability {
    /// Check if this capability requires explicit admin approval
    pub fn requires_approval(&self) -> bool {
        matches!(
            self,
            Capability::FinancialRead
                | Capability::FinancialWrite
                | Capability::TenantDataWrite
                | Capability::UserProfileWrite
        )
    }

    /// Check if this capability is dangerous (modifies critical state)
    pub fn is_dangerous(&self) -> bool {
        matches!(
            self,
            Capability::FinancialWrite | Capability::TenantDataWrite
        )
    }

    /// Get capability severity level (0-3)
    pub fn severity(&self) -> u8 {
        match self {
            // Critical - requires explicit approval
            Capability::FinancialWrite | Capability::TenantDataWrite => 3,
            // High - requires review
            Capability::FinancialRead | Capability::UserProfileWrite => 2,
            // Medium - device access
            Capability::Camera | Capability::Microphone | Capability::Geolocation => 1,
            // Low - safe
            _ => 0,
        }
    }
}

// ════════════════════════════════════════════════════════════════════════════
// Runtime Type
// ════════════════════════════════════════════════════════════════════════════

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Default, utoipa::ToSchema)]
#[serde(rename_all = "lowercase")]
pub enum RuntimeType {
    #[default]
    Frontend, // Svelte/JS frontend plugin
    Wasm,    // WASM backend plugin
    Service, // External service (HTTP)
    Hybrid,  // Both frontend + backend
}

// ════════════════════════════════════════════════════════════════════════════
// Plugin Status
// ════════════════════════════════════════════════════════════════════════════

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize, Default)]
#[serde(rename_all = "lowercase")]
pub enum PluginStatus {
    #[default]
    Installed,
    Enabled,
    Disabled,
    Error,
    Updating,
    PendingApproval, // For plugins with dangerous capabilities
}

// ════════════════════════════════════════════════════════════════════════════
// Full Plugin Entity (Database Record)
// ════════════════════════════════════════════════════════════════════════════

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Plugin {
    // ─── Identity ────────────────────────────────────────────────────────────
    pub id: Uuid,
    pub tenant_id: Option<Uuid>,
    pub plugin_id: String,
    pub name: String,
    pub version: String,
    pub description: Option<String>,

    // ─── Author ──────────────────────────────────────────────────────────────
    pub author: Option<String>,
    pub author_email: Option<String>,
    pub author_website: Option<String>,

    // ─── Display ─────────────────────────────────────────────────────────────
    pub icon: Option<String>,
    pub banner: Option<String>,
    pub category: Option<String>,
    pub tags: Vec<String>,

    // ─── Links ───────────────────────────────────────────────────────────────
    pub homepage_url: Option<String>,
    pub documentation_url: Option<String>,
    pub repository_url: Option<String>,
    pub support_url: Option<String>,

    // ─── Runtime ─────────────────────────────────────────────────────────────
    pub runtime: String,
    pub entry_point: Option<String>,
    pub capabilities: Vec<String>,
    pub permissions: Vec<String>,
    pub data_access: Vec<String>,
    pub network_access: Vec<String>,
    pub config: serde_json::Value,

    // ─── WASM Specifics ──────────────────────────────────────────────────────
    pub wasm_path: Option<String>,
    pub wasm_hash: Option<String>,
    pub wasm_size_bytes: Option<i64>,

    // ─── Trust & Verification ────────────────────────────────────────────────
    pub verified: bool,
    pub official: bool,
    pub featured: bool,
    pub is_core_plugin: bool,

    // ─── Compatibility ───────────────────────────────────────────────────────
    pub min_app_version: Option<String>,
    pub max_app_version: Option<String>,

    // ─── Status ──────────────────────────────────────────────────────────────
    pub status: String,
    pub is_active: bool,
    pub error_message: Option<String>,

    // ─── UI Extensions ───────────────────────────────────────────────────────
    pub ui: Option<serde_json::Value>,

    // ─── Timestamps ──────────────────────────────────────────────────────────
    pub installed_at: DateTime<Utc>,
    pub enabled_at: Option<DateTime<Utc>>,
    pub disabled_at: Option<DateTime<Utc>>,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}

impl Plugin {
    /// Create a new plugin from manifest
    pub fn from_manifest(manifest: &Manifest, tenant_id: Option<Uuid>) -> Self {
        let (author_name, author_email, author_website) = match &manifest.author {
            Some(a) => (Some(a.name.clone()), a.email.clone(), a.website.clone()),
            None => (None, None, None),
        };

        Self {
            id: Uuid::new_v4(),
            tenant_id,
            plugin_id: manifest.id.clone(),
            name: manifest.name.clone(),
            version: manifest.version.clone(),
            description: manifest.description.clone(),

            // Author
            author: author_name,
            author_email,
            author_website,

            // Display
            icon: manifest.icon.clone(),
            banner: manifest.banner.clone(),
            category: manifest.category.clone(),
            tags: manifest.tags.clone(),

            // Links
            homepage_url: manifest.homepage.clone(),
            documentation_url: manifest.documentation.clone(),
            repository_url: manifest.repository.clone(),
            support_url: manifest.support.clone(),

            // Runtime
            runtime: match manifest.runtime {
                RuntimeType::Frontend => "frontend".to_string(),
                RuntimeType::Wasm => "wasm".to_string(),
                RuntimeType::Service => "service".to_string(),
                RuntimeType::Hybrid => "hybrid".to_string(),
            },
            entry_point: manifest.entry.clone(),
            capabilities: manifest
                .capabilities
                .iter()
                .map(|c| {
                    serde_json::to_string(c)
                        .unwrap_or_default()
                        .trim_matches('"')
                        .to_string()
                })
                .collect(),
            permissions: manifest.permissions.clone(),
            data_access: manifest.data_access.clone(),
            network_access: manifest.network_access.clone(),
            config: serde_json::Value::Object(serde_json::Map::new()),

            // UI
            ui: manifest
                .ui
                .as_ref()
                .map(|ui| serde_json::to_value(ui).unwrap_or_default()),

            // WASM
            wasm_path: None,
            wasm_hash: None,
            wasm_size_bytes: None,

            // Trust
            verified: manifest.verified,
            official: manifest.official,
            featured: manifest.featured,
            is_core_plugin: manifest.is_core_plugin,

            // Compatibility
            min_app_version: manifest.min_app_version.clone(),
            max_app_version: manifest.max_app_version.clone(),

            // Status
            status: "installed".to_string(),
            is_active: false,
            error_message: None,

            // Timestamps
            installed_at: Utc::now(),
            enabled_at: None,
            disabled_at: None,
            created_at: Utc::now(),
            updated_at: Utc::now(),
        }
    }

    /// Get status as enum
    pub fn status_enum(&self) -> PluginStatus {
        match self.status.as_str() {
            "installed" => PluginStatus::Installed,
            "enabled" => PluginStatus::Enabled,
            "disabled" => PluginStatus::Disabled,
            "error" => PluginStatus::Error,
            "updating" => PluginStatus::Updating,
            "pending_approval" => PluginStatus::PendingApproval,
            _ => PluginStatus::Installed,
        }
    }

    /// Check if plugin has a specific capability
    pub fn has_capability(&self, cap: &Capability) -> bool {
        let cap_str = serde_json::to_string(cap)
            .unwrap_or_default()
            .trim_matches('"')
            .to_string();
        self.capabilities.contains(&cap_str)
    }

    /// Check if plugin has dangerous capabilities requiring approval
    pub fn requires_approval(&self) -> bool {
        self.capabilities
            .iter()
            .any(|c| matches!(c.as_str(), "financial_write" | "tenant_data_write"))
    }
}

// ════════════════════════════════════════════════════════════════════════════
// Plugin Event (Audit Log)
// ════════════════════════════════════════════════════════════════════════════

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PluginEvent {
    pub id: Uuid,
    pub plugin_id: Uuid,
    pub tenant_id: Uuid,
    pub event_type: String,
    pub event_data: serde_json::Value,
    pub created_at: DateTime<Utc>,
}

// ════════════════════════════════════════════════════════════════════════════
// Request DTOs
// ════════════════════════════════════════════════════════════════════════════

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct InstallPluginRequest {
    pub manifest: Manifest,
    pub wasm_bytes: Option<Vec<u8>>,
    pub config: Option<serde_json::Value>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct UpdatePluginConfigRequest {
    pub config: serde_json::Value,
}

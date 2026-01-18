// ════════════════════════════════════════════════════════════════════════════
// Plugin API Integration Tests
// ════════════════════════════════════════════════════════════════════════════

#[cfg(test)]
mod plugin_api_tests {
    use serde_json::json;
    use uuid::Uuid;

    /// Test manifest that matches pixco format
    fn sample_manifest() -> serde_json::Value {
        json!({
            "id": "test-plugin",
            "name": "Test Plugin",
            "version": "1.0.0",
            "description": "A test plugin for integration testing",
            "entry": "./index.js",
            "author": {
                "name": "Test Developer",
                "email": "test@example.com",
                "website": "https://example.com"
            },
            "icon": "🧪",
            "category": "Testing",
            "tags": ["test", "example"],
            "permissions": [],
            "dataAccess": [],
            "networkAccess": [],
            "runtime": "frontend",
            "capabilities": ["storage_read", "log_info"],
            "minAppVersion": "2.0.0",
            "verified": false,
            "official": false,
            "featured": false
        })
    }

    /// Test manifest with dangerous capabilities
    fn dangerous_manifest() -> serde_json::Value {
        json!({
            "id": "dangerous-plugin",
            "name": "Dangerous Plugin",
            "version": "1.0.0",
            "description": "Plugin with dangerous capabilities",
            "runtime": "wasm",
            "capabilities": ["financial_write", "tenant_data_write"]
        })
    }

    #[test]
    fn test_manifest_parsing() {
        use crate::modules::system::domain::plugin::Manifest;

        let manifest_json = sample_manifest();
        let result: Result<Manifest, _> = serde_json::from_value(manifest_json);

        assert!(
            result.is_ok(),
            "Failed to parse manifest: {:?}",
            result.err()
        );

        let manifest = result.unwrap();
        assert_eq!(manifest.id, "test-plugin");
        assert_eq!(manifest.name, "Test Plugin");
        assert_eq!(manifest.version, "1.0.0");
        assert!(manifest.author.is_some());
        assert_eq!(manifest.author.unwrap().name, "Test Developer");
    }

    #[test]
    fn test_ui_extensions_parsing() {
        use crate::modules::system::domain::plugin::entity::MenuExtension;
        use crate::modules::system::domain::plugin::Manifest;

        let ui_manifest = json!({
            "id": "ui-plugin",
            "name": "UI Plugin",
            "version": "1.0.0",
            "runtime": "frontend",
            "ui": {
                "menus": [
                    {
                        "id": "blog-dashboard",
                        "label": "plugin.ui-plugin.menu.dashboard", // i18n key with namespace
                        "path": "/blog",
                        "icon": "layout",
                        "parentId": "sidebar.main",
                        "order": 1,
                        "permissions": ["blog:read"]
                    },
                    {
                        "id": "blog-mobile-nav",
                        "label": "plugin.ui-plugin.menu.blog",
                        "path": "/blog",
                        "icon": "<svg>...</svg>", // Custom SVG
                        "parentId": "mobile.bottom_nav",
                        "order": 2
                    },
                    {
                        "id": "user-tabs-posts",
                        "label": "plugin.ui-plugin.tabs.posts",
                        "path": "/users/:id/posts",
                        "parentId": "pages.tabs", // Contextual zone
                        "order": 0
                    }
                ]
            }
        });

        let manifest: Manifest = serde_json::from_value(ui_manifest).unwrap();

        assert!(manifest.ui.is_some());
        let ui = manifest.ui.unwrap();
        assert_eq!(ui.menus.len(), 3);

        // Verify Web Sidebar Item
        let web_item = ui.menus.iter().find(|m| m.id == "blog-dashboard").unwrap();
        assert_eq!(web_item.label, "plugin.ui-plugin.menu.dashboard");
        assert_eq!(web_item.parent_id.as_deref(), Some("sidebar.main"));
        assert!(web_item.permissions.contains(&"blog:read".to_string()));

        // Verify Mobile Item with Custom SVG
        let mobile_item = ui.menus.iter().find(|m| m.id == "blog-mobile-nav").unwrap();
        assert_eq!(mobile_item.label, "plugin.ui-plugin.menu.blog");
        assert!(mobile_item.icon.as_ref().unwrap().starts_with("<svg>"));

        // Verify Contextual Tab
        let tab_item = ui.menus.iter().find(|m| m.id == "user-tabs-posts").unwrap();
        assert_eq!(tab_item.parent_id.as_deref(), Some("pages.tabs"));
    }

    #[test]
    fn test_plugin_from_manifest() {
        use crate::modules::system::domain::plugin::{Manifest, Plugin};

        let manifest_json = sample_manifest();
        let manifest: Manifest = serde_json::from_value(manifest_json).unwrap();

        let tenant_id = Uuid::new_v4();
        let plugin = Plugin::from_manifest(&manifest, Some(tenant_id));

        assert_eq!(plugin.plugin_id, "test-plugin");
        assert_eq!(plugin.name, "Test Plugin");
        assert!(plugin.tenant_id.is_some());
        assert_eq!(plugin.tenant_id.unwrap(), tenant_id);
        assert_eq!(plugin.status, "installed");
        assert!(!plugin.is_active);
    }

    #[test]
    fn test_dangerous_capability_detection() {
        use crate::modules::system::domain::plugin::{Capability, Manifest};

        let manifest_json = dangerous_manifest();
        let manifest: Manifest = serde_json::from_value(manifest_json).unwrap();

        // Check that dangerous capabilities are detected
        assert!(manifest.capabilities.iter().any(|c| c.is_dangerous()));

        // Should have FinancialWrite
        assert!(manifest.capabilities.contains(&Capability::FinancialWrite));
    }

    #[test]
    fn test_capability_approval_requirements() {
        use crate::modules::system::domain::plugin::Capability;

        // Safe capabilities
        assert!(!Capability::StorageRead.requires_approval());
        assert!(!Capability::LogInfo.requires_approval());
        assert!(!Capability::EventEmit.requires_approval());

        // Dangerous capabilities
        assert!(Capability::FinancialRead.requires_approval());
        assert!(Capability::FinancialWrite.requires_approval());
        assert!(Capability::TenantDataWrite.requires_approval());

        // Check is_dangerous
        assert!(Capability::FinancialWrite.is_dangerous());
        assert!(Capability::TenantDataWrite.is_dangerous());
        assert!(!Capability::FinancialRead.is_dangerous()); // read is not dangerous, just requires approval
    }

    #[test]
    fn test_security_summary_risk_levels() {
        use crate::modules::system::domain::plugin::Manifest;

        // Low risk - no special permissions
        let low_risk: Manifest = serde_json::from_value(json!({
            "id": "low-risk",
            "name": "Low Risk",
            "version": "1.0.0",
            "capabilities": ["log_info"]
        }))
        .unwrap();
        assert!(low_risk.capabilities.iter().all(|c| !c.requires_approval()));

        // Critical risk - dangerous capabilities
        let critical: Manifest = serde_json::from_value(json!({
            "id": "critical",
            "name": "Critical",
            "version": "1.0.0",
            "capabilities": ["financial_write"]
        }))
        .unwrap();
        assert!(critical.capabilities.iter().any(|c| c.is_dangerous()));
    }

    #[test]
    fn test_runtime_types() {
        use crate::modules::system::domain::plugin::entity::{Manifest, RuntimeType};

        let frontend: Manifest = serde_json::from_value(json!({
            "id": "frontend",
            "name": "Frontend",
            "version": "1.0.0",
            "runtime": "frontend"
        }))
        .unwrap();
        assert_eq!(frontend.runtime, RuntimeType::Frontend);

        let wasm: Manifest = serde_json::from_value(json!({
            "id": "wasm",
            "name": "WASM",
            "version": "1.0.0",
            "runtime": "wasm"
        }))
        .unwrap();
        assert_eq!(wasm.runtime, RuntimeType::Wasm);
    }

    #[test]
    fn test_plugin_visibility() {
        use crate::modules::system::domain::plugin::entity::{Manifest, PluginVisibility};

        // Test Shared Visibility
        let shared: Manifest = serde_json::from_value(json!({
            "id": "shared-plugin",
            "name": "Shared Plugin",
            "version": "1.0.0",
            "visibility": "shared"
        }))
        .unwrap();
        assert_eq!(shared.visibility, PluginVisibility::Shared);

        // Test Global Visibility
        let global: Manifest = serde_json::from_value(json!({
            "id": "global-plugin",
            "name": "Global Plugin",
            "version": "1.0.0",
            "visibility": "global"
        }))
        .unwrap();
        assert_eq!(global.visibility, PluginVisibility::Global);

        // Test Default (Private)
        let private: Manifest = serde_json::from_value(json!({
            "id": "private-plugin",
            "name": "Private Plugin",
            "version": "1.0.0"
        }))
        .unwrap();
        assert_eq!(private.visibility, PluginVisibility::Private);
    }
}

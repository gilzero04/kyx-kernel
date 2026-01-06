// ════════════════════════════════════════════════════════════════════════════
// Plugin Registry - Runtime Plugin Manager
// ════════════════════════════════════════════════════════════════════════════

use anyhow::{Result, anyhow};
use dashmap::DashMap;
use std::sync::Arc;
use uuid::Uuid;
use log::{info, warn};

use crate::modules::system::domain::plugin::{Plugin, Manifest, Capability};
use crate::modules::system::infrastructure::repositories::plugin::PluginRepository;
use crate::modules::system::infrastructure::wasm_engine::WasmEngine;
use crate::core::infrastructure::database::Database;
use crate::core::infrastructure::redis::Redis;

/// Loaded plugin with runtime instance
pub struct LoadedPlugin {
    pub plugin: Plugin,
    pub wasm_instance: Option<wasmer::Instance>,
}

/// Plugin Registry - manages plugin lifecycle
pub struct PluginRegistry {
    /// In-memory cache of loaded plugins (plugin_id -> LoadedPlugin)
    plugins: DashMap<String, LoadedPlugin>,
    /// Database repository
    repository: PluginRepository,
    /// WASM engine for loading plugins
    wasm_engine: WasmEngine,
    /// Redis for caching
    redis: Arc<Redis>,
}

impl PluginRegistry {
    /// Create a new plugin registry
    pub fn new(database: Arc<Database>, redis: Arc<Redis>) -> Self {
        Self {
            plugins: DashMap::new(),
            repository: PluginRepository::new(database.pool.clone()),
            wasm_engine: WasmEngine::new(),
            redis,
        }
    }
    
    /// Load all active plugins for a tenant into memory
    pub async fn load_tenant_plugins(&self, tenant_id: Uuid) -> Result<usize> {
        let plugins = self.repository.find_active_by_tenant(tenant_id).await?;
        let count = plugins.len();
        
        for plugin in plugins {
            let key = format!("{}:{}", tenant_id, plugin.plugin_id);
            
            // Try to load WASM if path exists
            let wasm_instance = if let Some(ref path) = plugin.wasm_path {
                match self.wasm_engine.load_from_path(path).await {
                    Ok(instance) => {
                        info!("Loaded WASM plugin: {} v{}", plugin.name, plugin.version);
                        Some(instance)
                    }
                    Err(e) => {
                        warn!("Failed to load WASM for plugin {}: {}", plugin.name, e);
                        None
                    }
                }
            } else {
                None
            };
            
            self.plugins.insert(key, LoadedPlugin { plugin, wasm_instance });
        }
        
        info!("Loaded {} plugins for tenant {}", count, tenant_id);
        Ok(count)
    }
    
    /// Get a loaded plugin by ID
    pub fn get(&self, tenant_id: Uuid, plugin_id: &str) -> Option<dashmap::mapref::one::Ref<'_, String, LoadedPlugin>> {
        let key = format!("{}:{}", tenant_id, plugin_id);
        self.plugins.get(&key)
    }
    
    /// Check if a plugin is loaded
    pub fn is_loaded(&self, tenant_id: Uuid, plugin_id: &str) -> bool {
        let key = format!("{}:{}", tenant_id, plugin_id);
        self.plugins.contains_key(&key)
    }
    
    /// List all plugins for a tenant (from DB)
    pub async fn list_plugins(&self, tenant_id: Uuid) -> Result<Vec<Plugin>> {
        self.repository.find_by_tenant(tenant_id).await
    }
    
    /// Get plugin by database ID
    pub async fn get_plugin(&self, id: Uuid) -> Result<Option<Plugin>> {
        self.repository.find_by_id(id).await
    }
    
    /// Install a new plugin
    pub async fn install(
        &self,
        tenant_id: Uuid,
        manifest: Manifest,
        wasm_bytes: Option<Vec<u8>>,
        config: Option<serde_json::Value>,
    ) -> Result<Plugin> {
        // Check for dangerous capabilities
        for cap in &manifest.capabilities {
            if cap.is_dangerous() {
                return Err(anyhow!(
                    "Plugin requests dangerous capability {:?}. Manual approval required.",
                    cap
                ));
            }
        }
        
        // Check if already installed
        if let Some(_) = self.repository.find_by_plugin_id(tenant_id, &manifest.id).await? {
            return Err(anyhow!("Plugin {} is already installed", manifest.id));
        }
        
        // Create plugin record
        let mut plugin = Plugin::from_manifest(&manifest, Some(tenant_id));
        
        // Set config if provided
        if let Some(cfg) = config {
            plugin.config = cfg;
        }
        
        // Save WASM bytes if provided
        if let Some(bytes) = wasm_bytes {
            let wasm_path = format!("plugins/{}/{}.wasm", tenant_id, manifest.id);
            // TODO: Save to storage
            plugin.wasm_path = Some(wasm_path);
            plugin.wasm_size_bytes = Some(bytes.len() as i64);
            plugin.wasm_hash = Some(simple_hash_hex(&bytes));
        }
        
        // Insert into database
        let installed = self.repository.insert(&plugin).await?;
        
        // Log event
        self.repository.log_event(
            installed.id,
            tenant_id,
            "installed",
            serde_json::json!({
                "version": installed.version,
                "capabilities": installed.capabilities
            })
        ).await?;
        
        info!("Installed plugin: {} v{} for tenant {}", installed.name, installed.version, tenant_id);
        
        Ok(installed)
    }
    
    /// Enable a plugin
    pub async fn enable(&self, tenant_id: Uuid, plugin_id: Uuid) -> Result<()> {
        let plugin = self.repository.find_by_id(plugin_id).await?
            .ok_or_else(|| anyhow!("Plugin not found"))?;
        
        // Verify tenant ownership
        if plugin.tenant_id != Some(tenant_id) {
            return Err(anyhow!("Plugin does not belong to this tenant"));
        }
        
        // Enable in database
        self.repository.enable(plugin_id).await?;
        
        // Load into memory
        let key = format!("{}:{}", tenant_id, plugin.plugin_id);
        let wasm_instance = if let Some(ref path) = plugin.wasm_path {
            self.wasm_engine.load_from_path(path).await.ok()
        } else {
            None
        };
        
        let mut updated_plugin = plugin.clone();
        updated_plugin.is_active = true;
        updated_plugin.status = "enabled".to_string();
        
        self.plugins.insert(key, LoadedPlugin { 
            plugin: updated_plugin, 
            wasm_instance 
        });
        
        // Log event
        self.repository.log_event(plugin_id, tenant_id, "enabled", serde_json::json!({})).await?;
        
        info!("Enabled plugin: {}", plugin.name);
        Ok(())
    }
    
    /// Disable a plugin
    pub async fn disable(&self, tenant_id: Uuid, plugin_id: Uuid) -> Result<()> {
        let plugin = self.repository.find_by_id(plugin_id).await?
            .ok_or_else(|| anyhow!("Plugin not found"))?;
        
        // Verify tenant ownership
        if plugin.tenant_id != Some(tenant_id) {
            return Err(anyhow!("Plugin does not belong to this tenant"));
        }
        
        // Disable in database
        self.repository.disable(plugin_id).await?;
        
        // Remove from memory
        let key = format!("{}:{}", tenant_id, plugin.plugin_id);
        self.plugins.remove(&key);
        
        // Log event
        self.repository.log_event(plugin_id, tenant_id, "disabled", serde_json::json!({})).await?;
        
        info!("Disabled plugin: {}", plugin.name);
        Ok(())
    }
    
    /// Uninstall a plugin
    pub async fn uninstall(&self, tenant_id: Uuid, plugin_id: Uuid) -> Result<()> {
        let plugin = self.repository.find_by_id(plugin_id).await?
            .ok_or_else(|| anyhow!("Plugin not found"))?;
        
        // Verify tenant ownership
        if plugin.tenant_id != Some(tenant_id) {
            return Err(anyhow!("Plugin does not belong to this tenant"));
        }
        
        // Remove from memory first
        let key = format!("{}:{}", tenant_id, plugin.plugin_id);
        self.plugins.remove(&key);
        
        // TODO: Delete WASM file from storage
        
        // Delete from database (cascades to events)
        self.repository.delete(plugin_id).await?;
        
        info!("Uninstalled plugin: {}", plugin.name);
        Ok(())
    }
    
    /// Update plugin configuration
    pub async fn update_config(&self, tenant_id: Uuid, plugin_id: Uuid, config: serde_json::Value) -> Result<()> {
        let plugin = self.repository.find_by_id(plugin_id).await?
            .ok_or_else(|| anyhow!("Plugin not found"))?;
        
        // Verify tenant ownership
        if plugin.tenant_id != Some(tenant_id) {
            return Err(anyhow!("Plugin does not belong to this tenant"));
        }
        
        self.repository.update_config(plugin_id, config.clone()).await?;
        
        // Update in memory if loaded
        let key = format!("{}:{}", tenant_id, plugin.plugin_id);
        if let Some(mut loaded) = self.plugins.get_mut(&key) {
            loaded.plugin.config = config;
        }
        
        Ok(())
    }
    
    /// Validate that a plugin has a required capability
    pub fn validate_capability(&self, tenant_id: Uuid, plugin_id: &str, required: &Capability) -> bool {
        if let Some(loaded) = self.get(tenant_id, plugin_id) {
            loaded.plugin.has_capability(required)
        } else {
            false
        }
    }
}

/// Helper: compute simple hash of bytes (using std only)
fn simple_hash_hex(data: &[u8]) -> String {
    use std::collections::hash_map::DefaultHasher;
    use std::hash::{Hash, Hasher};
    
    let mut hasher = DefaultHasher::new();
    data.hash(&mut hasher);
    let hash = hasher.finish();
    format!("{:016x}", hash)
}

// ════════════════════════════════════════════════════════════════════════════
// WASM Host Functions - Plugin↔Kernel Bindings
// ════════════════════════════════════════════════════════════════════════════
//
// These functions are exposed to WASM plugins as imports.
// They allow plugins to interact with the kernel safely.
//
// ════════════════════════════════════════════════════════════════════════════

use parking_lot::RwLock;
use std::collections::HashMap;
use std::sync::Arc;
use uuid::Uuid;

// ════════════════════════════════════════════════════════════════════════════
// Plugin Context (passed to host functions)
// ════════════════════════════════════════════════════════════════════════════

/// Context available to plugin during execution
#[derive(Clone)]
pub struct PluginContext {
    pub plugin_id: String,
    pub tenant_id: Uuid,
    pub capabilities: Vec<String>,
}

impl PluginContext {
    pub fn new(plugin_id: &str, tenant_id: Uuid, capabilities: Vec<String>) -> Self {
        Self {
            plugin_id: plugin_id.to_string(),
            tenant_id,
            capabilities,
        }
    }

    /// Check if plugin has a specific capability
    pub fn has_capability(&self, cap: &str) -> bool {
        self.capabilities.contains(&cap.to_string())
    }
}

// ════════════════════════════════════════════════════════════════════════════
// In-Memory Key-Value Store (per plugin)
// ════════════════════════════════════════════════════════════════════════════

/// Simple in-memory KV store for plugin data
/// In production, this would use Redis/PostgreSQL
pub struct PluginKvStore {
    data: RwLock<HashMap<String, HashMap<String, Vec<u8>>>>,
}

impl Default for PluginKvStore {
    fn default() -> Self {
        Self::new()
    }
}

impl PluginKvStore {
    pub fn new() -> Self {
        Self {
            data: RwLock::new(HashMap::new()),
        }
    }

    /// Get value for a key (scoped to plugin)
    pub fn get(&self, plugin_id: &str, key: &str) -> Option<Vec<u8>> {
        let data = self.data.read();
        data.get(plugin_id)
            .and_then(|plugin_data| plugin_data.get(key).cloned())
    }

    /// Set value for a key (scoped to plugin)
    pub fn set(&self, plugin_id: &str, key: &str, value: Vec<u8>) {
        let mut data = self.data.write();
        data.entry(plugin_id.to_string())
            .or_default()
            .insert(key.to_string(), value);
    }

    /// Delete a key (scoped to plugin)
    pub fn delete(&self, plugin_id: &str, key: &str) -> bool {
        let mut data = self.data.write();
        if let Some(plugin_data) = data.get_mut(plugin_id) {
            return plugin_data.remove(key).is_some();
        }
        false
    }

    /// List all keys for a plugin
    pub fn list_keys(&self, plugin_id: &str) -> Vec<String> {
        let data = self.data.read();
        data.get(plugin_id)
            .map(|plugin_data| plugin_data.keys().cloned().collect())
            .unwrap_or_default()
    }

    /// Clear all data for a plugin
    pub fn clear(&self, plugin_id: &str) {
        let mut data = self.data.write();
        data.remove(plugin_id);
    }
}

// ════════════════════════════════════════════════════════════════════════════
// Event Bus (pub/sub for plugins)
// ════════════════════════════════════════════════════════════════════════════

/// Event emitted by a plugin
#[derive(Clone, Debug)]
pub struct PluginEvent {
    pub plugin_id: String,
    pub tenant_id: Uuid,
    pub event_type: String,
    pub payload: serde_json::Value,
    pub timestamp: chrono::DateTime<chrono::Utc>,
}

/// Simple event bus for plugin events
pub struct PluginEventBus {
    events: RwLock<Vec<PluginEvent>>,
    subscribers: RwLock<HashMap<String, Vec<String>>>, // event_type -> [plugin_ids]
}

impl Default for PluginEventBus {
    fn default() -> Self {
        Self::new()
    }
}

impl PluginEventBus {
    pub fn new() -> Self {
        Self {
            events: RwLock::new(Vec::new()),
            subscribers: RwLock::new(HashMap::new()),
        }
    }

    /// Emit an event
    pub fn emit(
        &self,
        plugin_id: &str,
        tenant_id: Uuid,
        event_type: &str,
        payload: serde_json::Value,
    ) {
        let event = PluginEvent {
            plugin_id: plugin_id.to_string(),
            tenant_id,
            event_type: event_type.to_string(),
            payload,
            timestamp: chrono::Utc::now(),
        };

        log::debug!("Plugin {} emitted event: {}", plugin_id, event_type);

        let mut events = self.events.write();
        events.push(event);

        // Keep last 1000 events
        if events.len() > 1000 {
            events.drain(0..100);
        }
    }

    /// Subscribe to an event type
    pub fn subscribe(&self, plugin_id: &str, event_type: &str) {
        let mut subs = self.subscribers.write();
        subs.entry(event_type.to_string())
            .or_default()
            .push(plugin_id.to_string());
    }

    /// Unsubscribe from an event type
    pub fn unsubscribe(&self, plugin_id: &str, event_type: &str) {
        let mut subs = self.subscribers.write();
        if let Some(subscribers) = subs.get_mut(event_type) {
            subscribers.retain(|id| id != plugin_id);
        }
    }

    /// Get recent events for a plugin (based on subscriptions)
    pub fn get_events_for_plugin(&self, plugin_id: &str, limit: usize) -> Vec<PluginEvent> {
        let subs = self.subscribers.read();
        let events = self.events.read();

        // Find event types this plugin is subscribed to
        let subscribed_types: Vec<&String> = subs
            .iter()
            .filter(|(_, subscribers)| subscribers.contains(&plugin_id.to_string()))
            .map(|(event_type, _)| event_type)
            .collect();

        events
            .iter()
            .filter(|e| subscribed_types.contains(&&e.event_type))
            .rev()
            .take(limit)
            .cloned()
            .collect()
    }
}

// ════════════════════════════════════════════════════════════════════════════
// Host Functions Registry
// ════════════════════════════════════════════════════════════════════════════

/// Central registry for all host function implementations
pub struct HostFunctions {
    pub kv_store: Arc<PluginKvStore>,
    pub event_bus: Arc<PluginEventBus>,
}

impl Default for HostFunctions {
    fn default() -> Self {
        Self::new()
    }
}

impl HostFunctions {
    pub fn new() -> Self {
        Self {
            kv_store: Arc::new(PluginKvStore::new()),
            event_bus: Arc::new(PluginEventBus::new()),
        }
    }

    // ═══════════════════════════════════════════════════════════════════════
    // Storage Functions
    // ═══════════════════════════════════════════════════════════════════════

    /// kv_get(key: string) -> Option<bytes>
    pub fn kv_get(&self, ctx: &PluginContext, key: &str) -> Option<Vec<u8>> {
        if !ctx.has_capability("storage_read") && !ctx.has_capability("storage") {
            log::warn!(
                "Plugin {} denied: missing storage_read capability",
                ctx.plugin_id
            );
            return None;
        }
        self.kv_store.get(&ctx.plugin_id, key)
    }

    /// kv_set(key: string, value: bytes) -> bool
    pub fn kv_set(&self, ctx: &PluginContext, key: &str, value: Vec<u8>) -> bool {
        if !ctx.has_capability("storage_write") && !ctx.has_capability("storage") {
            log::warn!(
                "Plugin {} denied: missing storage_write capability",
                ctx.plugin_id
            );
            return false;
        }
        self.kv_store.set(&ctx.plugin_id, key, value);
        true
    }

    /// kv_delete(key: string) -> bool
    pub fn kv_delete(&self, ctx: &PluginContext, key: &str) -> bool {
        if !ctx.has_capability("storage_write") && !ctx.has_capability("storage") {
            log::warn!(
                "Plugin {} denied: missing storage_write capability",
                ctx.plugin_id
            );
            return false;
        }
        self.kv_store.delete(&ctx.plugin_id, key)
    }

    /// kv_list() -> Vec<string>
    pub fn kv_list(&self, ctx: &PluginContext) -> Vec<String> {
        if !ctx.has_capability("storage_read") && !ctx.has_capability("storage") {
            log::warn!(
                "Plugin {} denied: missing storage_read capability",
                ctx.plugin_id
            );
            return vec![];
        }
        self.kv_store.list_keys(&ctx.plugin_id)
    }

    // ═══════════════════════════════════════════════════════════════════════
    // Event Functions
    // ═══════════════════════════════════════════════════════════════════════

    /// emit_event(type: string, payload: json) -> bool
    pub fn emit_event(
        &self,
        ctx: &PluginContext,
        event_type: &str,
        payload: serde_json::Value,
    ) -> bool {
        if !ctx.has_capability("event_emit") && !ctx.has_capability("event") {
            log::warn!(
                "Plugin {} denied: missing event_emit capability",
                ctx.plugin_id
            );
            return false;
        }
        self.event_bus
            .emit(&ctx.plugin_id, ctx.tenant_id, event_type, payload);
        true
    }

    /// subscribe_event(type: string) -> bool
    pub fn subscribe_event(&self, ctx: &PluginContext, event_type: &str) -> bool {
        if !ctx.has_capability("event_subscribe") && !ctx.has_capability("event") {
            log::warn!(
                "Plugin {} denied: missing event_subscribe capability",
                ctx.plugin_id
            );
            return false;
        }
        self.event_bus.subscribe(&ctx.plugin_id, event_type);
        true
    }

    /// get_events(limit: usize) -> Vec<PluginEvent>
    pub fn get_events(&self, ctx: &PluginContext, limit: usize) -> Vec<PluginEvent> {
        if !ctx.has_capability("event_subscribe") && !ctx.has_capability("event") {
            return vec![];
        }
        self.event_bus.get_events_for_plugin(&ctx.plugin_id, limit)
    }

    // ═══════════════════════════════════════════════════════════════════════
    // Logging Functions
    // ═══════════════════════════════════════════════════════════════════════

    /// log_info(message: string)
    pub fn log_info(&self, ctx: &PluginContext, message: &str) {
        if ctx.has_capability("log_info") {
            log::info!("[Plugin:{}] {}", ctx.plugin_id, message);
        }
    }

    /// log_warn(message: string)
    pub fn log_warn(&self, ctx: &PluginContext, message: &str) {
        if ctx.has_capability("log_warn") {
            log::warn!("[Plugin:{}] {}", ctx.plugin_id, message);
        }
    }

    /// log_error(message: string)
    pub fn log_error(&self, ctx: &PluginContext, message: &str) {
        if ctx.has_capability("log_error") {
            log::error!("[Plugin:{}] {}", ctx.plugin_id, message);
        }
    }

    // ═══════════════════════════════════════════════════════════════════════
    // Cleanup
    // ═══════════════════════════════════════════════════════════════════════

    /// Clean up all data for a plugin (called on uninstall)
    pub fn cleanup_plugin(&self, plugin_id: &str) {
        self.kv_store.clear(plugin_id);
        // Note: Event subscriptions will be cleaned up automatically on next check
        log::info!("Cleaned up data for plugin: {}", plugin_id);
    }
}

// ════════════════════════════════════════════════════════════════════════════
// Tests
// ════════════════════════════════════════════════════════════════════════════

#[cfg(test)]
mod tests {
    use super::*;

    fn test_context() -> PluginContext {
        PluginContext::new(
            "test-plugin",
            Uuid::new_v4(),
            vec![
                "storage".to_string(),
                "event".to_string(),
                "log_info".to_string(),
            ],
        )
    }

    #[test]
    fn test_kv_store() {
        let hf = HostFunctions::new();
        let ctx = test_context();

        // Set and get
        assert!(hf.kv_set(&ctx, "key1", b"value1".to_vec()));
        assert_eq!(hf.kv_get(&ctx, "key1"), Some(b"value1".to_vec()));

        // List keys
        let keys = hf.kv_list(&ctx);
        assert!(keys.contains(&"key1".to_string()));

        // Delete
        assert!(hf.kv_delete(&ctx, "key1"));
        assert_eq!(hf.kv_get(&ctx, "key1"), None);
    }

    #[test]
    fn test_event_bus() {
        let hf = HostFunctions::new();
        let ctx = test_context();

        // Subscribe and emit
        assert!(hf.subscribe_event(&ctx, "user.created"));
        assert!(hf.emit_event(&ctx, "user.created", serde_json::json!({"id": 123})));

        // Get events
        let events = hf.get_events(&ctx, 10);
        assert_eq!(events.len(), 1);
        assert_eq!(events[0].event_type, "user.created");
    }

    #[test]
    fn test_capability_check() {
        let hf = HostFunctions::new();
        let restricted_ctx = PluginContext::new("restricted", Uuid::new_v4(), vec![]);

        // Should fail without capability
        assert!(!hf.kv_set(&restricted_ctx, "key", b"value".to_vec()));
        assert!(!hf.emit_event(&restricted_ctx, "test", serde_json::json!({})));
    }
}

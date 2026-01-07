// ════════════════════════════════════════════════════════════════════════════
// Plugin Hooks - Event Registration and Dispatch
// ════════════════════════════════════════════════════════════════════════════

use std::collections::HashMap;
use std::future::Future;
use std::pin::Pin;
use std::sync::Arc;
use tokio::sync::RwLock;
use uuid::Uuid;
use log::debug;

/// Hook events that plugins can subscribe to
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum PluginHook {
    // Auth events
    OnUserLogin,
    OnUserLogout,
    OnUserSignup,
    
    // Tenant events
    OnTenantCreated,
    OnTenantUpdated,
    
    // Plan events (for kyx-plan plugin)
    OnPlanChanged,
    OnQuotaExceeded,
    
    // Payment events (for kyx-payment plugin)
    OnPaymentReceived,
    OnSubscriptionCreated,
    OnSubscriptionCancelled,
    
    // Notification events (for kyx-signal plugin)
    OnNotificationSent,
    
    // Custom hook (plugin-defined)
    Custom,
}

/// Context passed to hook handlers
#[derive(Debug, Clone)]
pub struct HookContext {
    pub tenant_id: Uuid,
    pub user_id: Option<Uuid>,
    pub event_type: PluginHook,
    pub data: serde_json::Value,
}

/// Async hook handler type
pub type HookHandler = Arc<
    dyn Fn(HookContext) -> Pin<Box<dyn Future<Output = Result<(), anyhow::Error>> + Send>>
        + Send
        + Sync
>;

/// Plugin hook registry - manages event subscriptions
pub struct HookRegistry {
    /// Subscribers by hook type: hook -> Vec<(plugin_id, handler)>
    subscribers: RwLock<HashMap<PluginHook, Vec<(String, HookHandler)>>>,
}

impl HookRegistry {
    pub fn new() -> Self {
        Self {
            subscribers: RwLock::new(HashMap::new()),
        }
    }
    
    /// Subscribe a plugin to a hook event
    pub async fn subscribe(
        &self,
        hook: PluginHook,
        plugin_id: impl Into<String>,
        handler: HookHandler,
    ) {
        let plugin_id = plugin_id.into();
        let mut subs = self.subscribers.write().await;
        
        subs.entry(hook)
            .or_insert_with(Vec::new)
            .push((plugin_id.clone(), handler));
        
        debug!("Plugin {} subscribed to {:?}", plugin_id, hook);
    }
    
    /// Unsubscribe a plugin from all hooks
    pub async fn unsubscribe_all(&self, plugin_id: &str) {
        let mut subs = self.subscribers.write().await;
        
        for handlers in subs.values_mut() {
            handlers.retain(|(id, _)| id != plugin_id);
        }
        
        debug!("Plugin {} unsubscribed from all hooks", plugin_id);
    }
    
    /// Dispatch a hook event to all subscribers
    /// Errors from handlers are logged but don't stop dispatch
    pub async fn dispatch(&self, context: HookContext) {
        let subs = self.subscribers.read().await;
        
        if let Some(handlers) = subs.get(&context.event_type) {
            debug!("Dispatching {:?} to {} handlers", context.event_type, handlers.len());
            
            for (plugin_id, handler) in handlers {
                let ctx = context.clone();
                if let Err(e) = handler(ctx).await {
                    log::warn!("Hook handler error for plugin {}: {}", plugin_id, e);
                }
            }
        }
    }
    
    /// Get count of subscribers for a hook
    pub async fn subscriber_count(&self, hook: PluginHook) -> usize {
        let subs = self.subscribers.read().await;
        subs.get(&hook).map(|v| v.len()).unwrap_or(0)
    }
}

impl Default for HookRegistry {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::sync::atomic::{AtomicUsize, Ordering};
    
    #[tokio::test]
    async fn test_hook_subscription_and_dispatch() {
        let registry = HookRegistry::new();
        let counter = Arc::new(AtomicUsize::new(0));
        let counter_clone = counter.clone();
        
        // Create handler
        let handler: HookHandler = Arc::new(move |_ctx| {
            let c = counter_clone.clone();
            Box::pin(async move {
                c.fetch_add(1, Ordering::SeqCst);
                Ok(())
            })
        });
        
        // Subscribe
        registry.subscribe(PluginHook::OnUserLogin, "test-plugin", handler).await;
        
        assert_eq!(registry.subscriber_count(PluginHook::OnUserLogin).await, 1);
        
        // Dispatch
        let ctx = HookContext {
            tenant_id: Uuid::new_v4(),
            user_id: Some(Uuid::new_v4()),
            event_type: PluginHook::OnUserLogin,
            data: serde_json::json!({"test": true}),
        };
        
        registry.dispatch(ctx).await;
        
        assert_eq!(counter.load(Ordering::SeqCst), 1);
    }
    
    #[tokio::test]
    async fn test_unsubscribe_all() {
        let registry = HookRegistry::new();
        
        let handler: HookHandler = Arc::new(|_| Box::pin(async { Ok(()) }));
        
        registry.subscribe(PluginHook::OnUserLogin, "plugin-1", handler.clone()).await;
        registry.subscribe(PluginHook::OnPaymentReceived, "plugin-1", handler.clone()).await;
        
        assert_eq!(registry.subscriber_count(PluginHook::OnUserLogin).await, 1);
        assert_eq!(registry.subscriber_count(PluginHook::OnPaymentReceived).await, 1);
        
        registry.unsubscribe_all("plugin-1").await;
        
        assert_eq!(registry.subscriber_count(PluginHook::OnUserLogin).await, 0);
        assert_eq!(registry.subscriber_count(PluginHook::OnPaymentReceived).await, 0);
    }
}

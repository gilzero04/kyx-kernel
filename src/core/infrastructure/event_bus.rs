use crate::core::domain::event_bus::EventBus;
use async_trait::async_trait;
use anyhow::Result;
use serde::Serialize;
use tokio::sync::broadcast;
use std::collections::HashMap;
use std::sync::Arc;
use tokio::sync::Mutex;

pub struct InMemoryEventBus {
    // Mapping from topic to broadcast sender
    channels: Arc<Mutex<HashMap<String, broadcast::Sender<String>>>>,
}

impl InMemoryEventBus {
    #[allow(dead_code)]
    pub fn new() -> Self {
        Self {
            channels: Arc::new(Mutex::new(HashMap::new())),
        }
    }
}

#[async_trait]
impl EventBus for InMemoryEventBus {
    async fn publish<E>(&self, topic: &str, event: E) -> Result<()>
    where
        E: Serialize + Send + Sync + 'static,
    {
        let event_json = serde_json::to_string(&event)?;
        let mut channels = self.channels.lock().await;
        
        if let Some(sender) = channels.get(topic) {
            let _ = sender.send(event_json);
        } else {
            // If no channel exists, create one (lazy initialization)
            let (tx, _) = broadcast::channel(100);
            let _ = tx.send(event_json);
            channels.insert(topic.to_string(), tx);
        }
        
        Ok(())
    }

    fn subscribe(&self, topic: &str) {
        // Implementation of subscription would return a receiver
        // Will be refined as we implement module-specific handlers
        println!("[EventBus] Subscription placeholder for topic: {}", topic);
    }
}

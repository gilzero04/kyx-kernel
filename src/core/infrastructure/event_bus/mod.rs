use crate::core::domain::event_bus::EventBus;
use anyhow::Result;
use async_trait::async_trait;
use serde::Serialize;
use std::collections::HashMap;
use std::sync::Arc;
use tokio::sync::Mutex;
use tokio::sync::broadcast;

pub struct InMemoryEventBus {
    // Mapping from topic to broadcast sender
    #[allow(dead_code)]
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
            // Capacity 1000 for backpressure - drops oldest if full
            let (tx, _) = broadcast::channel(1000);
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

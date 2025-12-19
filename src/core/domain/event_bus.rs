use async_trait::async_trait;
use anyhow::Result;
use serde::Serialize;

#[async_trait]
pub trait EventBus: Send + Sync {
    /// Publish an event to the bus
    async fn publish<E>(&self, topic: &str, event: E) -> Result<()>
    where
        E: Serialize + Send + Sync + 'static;

    /// Subscribe to a topic (placeholder for now, will implement handler registration)
    fn subscribe(&self, topic: &str);
}

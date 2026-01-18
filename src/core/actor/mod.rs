// ═══════════════════════════════════════════════════════════════════════════════
// Actor System — Base Traits and Utilities
// ═══════════════════════════════════════════════════════════════════════════════
// This module provides the foundation for the Actor model in Kyx Kernel.
// Actors are isolated units of computation that:
// - Process messages sequentially
// - Maintain their own state
// - Communicate only via message passing
// - Can be sharded by tenant_id for horizontal scaling
// ═══════════════════════════════════════════════════════════════════════════════

pub mod permission;
pub mod tenant;

use async_trait::async_trait;
use serde::{Deserialize, Serialize};
use std::sync::Arc;
use tokio::sync::mpsc;
use uuid::Uuid;

use crate::core::event::{CoreEvent, EventEnvelope};

// ═══════════════════════════════════════════════════════════════════════════════
// Actor Message Types
// ═══════════════════════════════════════════════════════════════════════════════

/// Message that can be sent to an actor
#[derive(Debug, Clone)]
pub enum ActorMessage {
    /// Process an event
    Event(EventEnvelope),
    /// Request actor to stop
    Stop,
    /// Health check
    Ping,
}

/// Response from an actor
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum ActorResponse {
    /// Acknowledgement
    Ack,
    /// Error with message
    Error(String),
    /// Health check response
    Pong,
    /// Actor is busy
    Busy,
}

// ═══════════════════════════════════════════════════════════════════════════════
// Actor Context
// ═══════════════════════════════════════════════════════════════════

/// Context passed to actor for each message
#[derive(Debug, Clone)]
pub struct ActorContext {
    /// The actor's unique ID
    pub actor_id: Uuid,
    /// Tenant ID this actor is responsible for (for sharding)
    pub tenant_id: Option<Uuid>,
    /// Actor name for logging
    pub name: String,
}

impl ActorContext {
    pub fn new(name: impl Into<String>) -> Self {
        Self {
            actor_id: Uuid::new_v4(),
            tenant_id: None,
            name: name.into(),
        }
    }

    pub fn with_tenant(mut self, tenant_id: Uuid) -> Self {
        self.tenant_id = Some(tenant_id);
        self
    }
}

// ═══════════════════════════════════════════════════════════════════════════════
// Actor Trait
// ═══════════════════════════════════════════════════════════════════════════════

/// Base trait for all actors
#[async_trait]
pub trait Actor: Send + Sync + 'static {
    /// Actor name for identification and logging
    fn name(&self) -> &str;

    /// Handle incoming event
    async fn handle_event(&mut self, ctx: &ActorContext, event: &CoreEvent) -> ActorResponse;

    /// Called when actor starts
    async fn on_start(&mut self, _ctx: &ActorContext) {
        log::info!("[{}] Actor started", self.name());
    }

    /// Called when actor stops
    async fn on_stop(&mut self, _ctx: &ActorContext) {
        log::info!("[{}] Actor stopped", self.name());
    }

    /// Handle ping (health check)
    async fn on_ping(&self, _ctx: &ActorContext) -> ActorResponse {
        ActorResponse::Pong
    }
}

// ═══════════════════════════════════════════════════════════════════════════════
// Actor Handle (for sending messages)
// ═══════════════════════════════════════════════════════════════════════════════

/// Handle to communicate with a running actor
#[derive(Clone)]
pub struct ActorHandle {
    sender: mpsc::Sender<ActorMessage>,
    ctx: ActorContext,
}

impl ActorHandle {
    /// Send an event to the actor
    pub async fn send_event(&self, envelope: EventEnvelope) -> Result<(), String> {
        self.sender
            .send(ActorMessage::Event(envelope))
            .await
            .map_err(|e| format!("Failed to send event: {}", e))
    }

    /// Send a ping to check if actor is alive
    pub async fn ping(&self) -> Result<(), String> {
        self.sender
            .send(ActorMessage::Ping)
            .await
            .map_err(|e| format!("Failed to ping: {}", e))
    }

    /// Request actor to stop
    pub async fn stop(&self) -> Result<(), String> {
        self.sender
            .send(ActorMessage::Stop)
            .await
            .map_err(|e| format!("Failed to stop: {}", e))
    }

    /// Get actor context
    pub fn context(&self) -> &ActorContext {
        &self.ctx
    }
}

// ═══════════════════════════════════════════════════════════════════════════════
// Actor Runner
// ═══════════════════════════════════════════════════════════════════════════════

/// Spawn and run an actor
pub struct ActorRunner;

impl ActorRunner {
    /// Spawn a new actor and return a handle to it
    pub fn spawn<A: Actor>(mut actor: A, ctx: ActorContext) -> ActorHandle {
        let (tx, mut rx) = mpsc::channel::<ActorMessage>(100);
        let ctx_clone = ctx.clone();

        tokio::spawn(async move {
            actor.on_start(&ctx_clone).await;

            while let Some(msg) = rx.recv().await {
                match msg {
                    ActorMessage::Event(envelope) => {
                        let _response = actor.handle_event(&ctx_clone, &envelope.event).await;
                    }
                    ActorMessage::Ping => {
                        let _response = actor.on_ping(&ctx_clone).await;
                    }
                    ActorMessage::Stop => {
                        actor.on_stop(&ctx_clone).await;
                        break;
                    }
                }
            }

            log::info!("[{}] Actor exited", actor.name());
        });

        ActorHandle { sender: tx, ctx }
    }

    /// Spawn a tenant-specific actor
    pub fn spawn_for_tenant<A: Actor>(actor: A, tenant_id: Uuid) -> ActorHandle {
        let ctx = ActorContext::new(format!("tenant-{}", tenant_id)).with_tenant(tenant_id);
        Self::spawn(actor, ctx)
    }
}

// ═══════════════════════════════════════════════════════════════════════════════
// Actor Registry (for managing multiple actors)
// ═══════════════════════════════════════════════════════════════════════════════

use std::collections::HashMap;
use tokio::sync::RwLock;

/// Registry for managing actor handles
pub struct ActorRegistry {
    actors: Arc<RwLock<HashMap<String, ActorHandle>>>,
}

impl ActorRegistry {
    pub fn new() -> Self {
        Self {
            actors: Arc::new(RwLock::new(HashMap::new())),
        }
    }

    /// Register an actor handle
    pub async fn register(&self, key: impl Into<String>, handle: ActorHandle) {
        let mut actors = self.actors.write().await;
        actors.insert(key.into(), handle);
    }

    /// Get an actor handle by key
    pub async fn get(&self, key: &str) -> Option<ActorHandle> {
        let actors = self.actors.read().await;
        actors.get(key).cloned()
    }

    /// Get or create a tenant actor
    pub async fn get_or_create_tenant<F, A>(&self, tenant_id: Uuid, factory: F) -> ActorHandle
    where
        F: FnOnce() -> A,
        A: Actor,
    {
        let key = format!("tenant-{}", tenant_id);

        // Check if exists
        {
            let actors = self.actors.read().await;
            if let Some(handle) = actors.get(&key) {
                return handle.clone();
            }
        }

        // Create new
        let actor = factory();
        let handle = ActorRunner::spawn_for_tenant(actor, tenant_id);

        let mut actors = self.actors.write().await;
        actors.insert(key, handle.clone());

        handle
    }

    /// Remove an actor
    pub async fn remove(&self, key: &str) -> Option<ActorHandle> {
        let mut actors = self.actors.write().await;
        actors.remove(key)
    }

    /// Stop all actors
    pub async fn stop_all(&self) {
        let actors = self.actors.read().await;
        for (_, handle) in actors.iter() {
            let _ = handle.stop().await;
        }
    }

    /// Get count of registered actors
    pub async fn count(&self) -> usize {
        self.actors.read().await.len()
    }
}

impl Default for ActorRegistry {
    fn default() -> Self {
        Self::new()
    }
}

// ═══════════════════════════════════════════════════════════════════════════════
// Tests
// ═══════════════════════════════════════════════════════════════════════════════

#[cfg(test)]
mod tests {
    use super::*;
    use crate::core::event::{CoreEventV1, EventSource, TenantEvent};

    struct TestActor {
        event_count: u32,
    }

    #[async_trait]
    impl Actor for TestActor {
        fn name(&self) -> &str {
            "test-actor"
        }

        async fn handle_event(&mut self, _ctx: &ActorContext, _event: &CoreEvent) -> ActorResponse {
            self.event_count += 1;
            ActorResponse::Ack
        }
    }

    #[tokio::test]
    async fn test_actor_spawn() {
        let actor = TestActor { event_count: 0 };
        let ctx = ActorContext::new("test");
        let handle = ActorRunner::spawn(actor, ctx);

        // Send ping
        handle.ping().await.unwrap();

        // Stop
        handle.stop().await.unwrap();
    }

    #[tokio::test]
    async fn test_actor_registry() {
        let registry = ActorRegistry::new();

        let actor = TestActor { event_count: 0 };
        let tenant_id = Uuid::new_v4();

        let handle = registry.get_or_create_tenant(tenant_id, || actor).await;
        assert!(handle.ping().await.is_ok());

        assert_eq!(registry.count().await, 1);

        registry.stop_all().await;
    }
}

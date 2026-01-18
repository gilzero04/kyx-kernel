// ═══════════════════════════════════════════════════════════════════════════════
// Tenant Actor — Per-Tenant State Management
// ═══════════════════════════════════════════════════════════════════════════════
// Each tenant can have its own actor instance for:
// - Isolated state management
// - Per-tenant event processing
// - Horizontal scaling (shard by tenant_id)
// ═══════════════════════════════════════════════════════════════════════════════

use async_trait::async_trait;
use uuid::Uuid;

use super::{Actor, ActorContext, ActorResponse};
use crate::core::event::{CoreEvent, CoreEventV1, TenantEvent, UserEvent};

/// Tenant-specific actor for handling tenant events
pub struct TenantActor {
    tenant_id: Uuid,
    tenant_name: String,
    // Statistics
    events_processed: u64,
    users_created: u32,
    users_deleted: u32,
}

impl TenantActor {
    pub fn new(tenant_id: Uuid, tenant_name: impl Into<String>) -> Self {
        Self {
            tenant_id,
            tenant_name: tenant_name.into(),
            events_processed: 0,
            users_created: 0,
            users_deleted: 0,
        }
    }

    /// Get tenant ID
    pub fn tenant_id(&self) -> Uuid {
        self.tenant_id
    }

    /// Get statistics
    pub fn stats(&self) -> TenantActorStats {
        TenantActorStats {
            tenant_id: self.tenant_id,
            tenant_name: self.tenant_name.clone(),
            events_processed: self.events_processed,
            users_created: self.users_created,
            users_deleted: self.users_deleted,
        }
    }

    /// Handle tenant-specific events
    fn handle_tenant_event(&mut self, event: &TenantEvent) {
        match event {
            TenantEvent::Created {
                tenant_id, name, ..
            } => {
                log::info!(
                    "[TenantActor:{}] Tenant created: {} ({})",
                    self.tenant_id,
                    name,
                    tenant_id
                );
            }
            TenantEvent::Updated { tenant_id, .. } => {
                log::info!(
                    "[TenantActor:{}] Tenant updated: {}",
                    self.tenant_id,
                    tenant_id
                );
            }
            TenantEvent::Deleted { tenant_id } => {
                log::info!(
                    "[TenantActor:{}] Tenant deleted: {}",
                    self.tenant_id,
                    tenant_id
                );
            }
            TenantEvent::BrandingUpdated { tenant_id, context } => {
                log::info!(
                    "[TenantActor:{}] Branding updated for {} in context {}",
                    self.tenant_id,
                    tenant_id,
                    context
                );
            }
        }
    }

    /// Handle user events within this tenant
    fn handle_user_event(&mut self, event: &UserEvent) {
        match event {
            UserEvent::Created {
                user_id,
                email,
                tenant_id,
            } => {
                if *tenant_id == self.tenant_id {
                    self.users_created += 1;
                    log::info!(
                        "[TenantActor:{}] User created: {} ({})",
                        self.tenant_id,
                        email,
                        user_id
                    );
                }
            }
            UserEvent::Deleted { user_id } => {
                self.users_deleted += 1;
                log::info!("[TenantActor:{}] User deleted: {}", self.tenant_id, user_id);
            }
            UserEvent::LoggedIn {
                user_id, tenant_id, ..
            } => {
                if *tenant_id == self.tenant_id {
                    log::debug!(
                        "[TenantActor:{}] User logged in: {}",
                        self.tenant_id,
                        user_id
                    );
                }
            }
            _ => {}
        }
    }
}

#[async_trait]
impl Actor for TenantActor {
    fn name(&self) -> &str {
        "TenantActor"
    }

    async fn handle_event(&mut self, _ctx: &ActorContext, event: &CoreEvent) -> ActorResponse {
        self.events_processed += 1;

        match event {
            CoreEvent::V1(v1) => match v1 {
                CoreEventV1::Tenant(tenant_event) => {
                    self.handle_tenant_event(tenant_event);
                }
                CoreEventV1::User(user_event) => {
                    self.handle_user_event(user_event);
                }
                _ => {
                    // Other events not handled by TenantActor
                }
            },
        }

        ActorResponse::Ack
    }

    async fn on_start(&mut self, ctx: &ActorContext) {
        log::info!(
            "[TenantActor:{}] Started for tenant '{}'",
            ctx.actor_id,
            self.tenant_name
        );
    }

    async fn on_stop(&mut self, ctx: &ActorContext) {
        log::info!(
            "[TenantActor:{}] Stopped. Processed {} events",
            ctx.actor_id,
            self.events_processed
        );
    }
}

/// Statistics for a tenant actor
#[derive(Debug, Clone)]
pub struct TenantActorStats {
    pub tenant_id: Uuid,
    pub tenant_name: String,
    pub events_processed: u64,
    pub users_created: u32,
    pub users_deleted: u32,
}

// ═══════════════════════════════════════════════════════════════════════════════
// Tests
// ═══════════════════════════════════════════════════════════════════════════════

#[cfg(test)]
mod tests {
    use super::*;
    use crate::core::actor::{ActorContext, ActorRunner};
    use crate::core::event::{CoreEventV1, EventEnvelope, EventSource, TenantEvent};

    #[tokio::test]
    async fn test_tenant_actor_creation() {
        let tenant_id = Uuid::new_v4();
        let actor = TenantActor::new(tenant_id, "Test Tenant");

        assert_eq!(actor.tenant_id(), tenant_id);
        assert_eq!(actor.stats().events_processed, 0);
    }

    #[tokio::test]
    async fn test_tenant_actor_handles_events() {
        let tenant_id = Uuid::new_v4();
        let actor = TenantActor::new(tenant_id, "Test Tenant");
        let handle = ActorRunner::spawn_for_tenant(actor, tenant_id);

        // Send tenant created event
        let event = EventEnvelope::internal(CoreEventV1::Tenant(TenantEvent::Created {
            tenant_id,
            name: "New Tenant".to_string(),
            slug: "new-tenant".to_string(),
            parent_id: None,
        }));

        handle.send_event(event).await.unwrap();

        // Give some time for processing
        tokio::time::sleep(tokio::time::Duration::from_millis(50)).await;

        handle.stop().await.unwrap();
    }
}

// ═══════════════════════════════════════════════════════════════════════════════
// Permission Actor — Cached Permission Checking with Actor Model
// ═══════════════════════════════════════════════════════════════════════════════
// Combines the Actor model with permission caching for:
// - Reduced database queries
// - Per-tenant permission state
// - Async permission resolution
// ═══════════════════════════════════════════════════════════════════════════════

use async_trait::async_trait;
use std::collections::HashMap;
use std::time::{Duration, Instant};
use uuid::Uuid;

use super::{Actor, ActorContext, ActorResponse};
use crate::core::event::{CoreEvent, CoreEventV1, UserEvent};

/// Permission actor for caching and resolving permissions
pub struct PermissionActor {
    tenant_id: Uuid,
    /// Cache: (user_id, permission) -> (has_permission, expires_at)
    cache: HashMap<(Uuid, String), (bool, Instant)>,
    /// TTL for cache entries
    ttl: Duration,
    /// Stats
    cache_hits: u64,
    cache_misses: u64,
    invalidations: u64,
}

impl PermissionActor {
    pub fn new(tenant_id: Uuid) -> Self {
        Self::with_ttl(tenant_id, Duration::from_secs(300)) // 5 min default
    }

    pub fn with_ttl(tenant_id: Uuid, ttl: Duration) -> Self {
        Self {
            tenant_id,
            cache: HashMap::new(),
            ttl,
            cache_hits: 0,
            cache_misses: 0,
            invalidations: 0,
        }
    }

    /// Check if user has permission (from cache)
    pub fn check_cached(&mut self, user_id: Uuid, permission: &str) -> Option<bool> {
        let key = (user_id, permission.to_string());

        if let Some((has_perm, expires_at)) = self.cache.get(&key) {
            if Instant::now() < *expires_at {
                self.cache_hits += 1;
                return Some(*has_perm);
            }
            // Expired, remove
            self.cache.remove(&key);
        }

        self.cache_misses += 1;
        None
    }

    /// Cache a permission check result
    pub fn cache_permission(&mut self, user_id: Uuid, permission: &str, has_permission: bool) {
        let key = (user_id, permission.to_string());
        let expires_at = Instant::now() + self.ttl;
        self.cache.insert(key, (has_permission, expires_at));
    }

    /// Invalidate all permissions for a user
    pub fn invalidate_user(&mut self, user_id: Uuid) {
        self.cache.retain(|(uid, _), _| *uid != user_id);
        self.invalidations += 1;
    }

    /// Invalidate all cached permissions
    pub fn invalidate_all(&mut self) {
        self.cache.clear();
        self.invalidations += 1;
    }

    /// Get stats
    pub fn stats(&self) -> PermissionActorStats {
        let total = self.cache_hits + self.cache_misses;
        PermissionActorStats {
            tenant_id: self.tenant_id,
            cache_size: self.cache.len(),
            cache_hits: self.cache_hits,
            cache_misses: self.cache_misses,
            hit_rate: if total > 0 {
                (self.cache_hits as f64 / total as f64) * 100.0
            } else {
                0.0
            },
            invalidations: self.invalidations,
        }
    }

    /// Handle user events that affect permissions
    fn handle_user_event(&mut self, event: &UserEvent) {
        match event {
            UserEvent::Deleted { user_id } => {
                self.invalidate_user(*user_id);
                log::info!(
                    "[PermissionActor:{}] Invalidated permissions for deleted user {}",
                    self.tenant_id,
                    user_id
                );
            }
            UserEvent::RoleAssigned { user_id, .. } => {
                self.invalidate_user(*user_id);
                log::info!(
                    "[PermissionActor:{}] Invalidated permissions for user {} (role change)",
                    self.tenant_id,
                    user_id
                );
            }
            UserEvent::PasswordChanged { user_id } => {
                // Password change might trigger re-auth, invalidate
                self.invalidate_user(*user_id);
            }
            _ => {}
        }
    }

    /// Cleanup expired entries
    fn cleanup_expired(&mut self) {
        let now = Instant::now();
        self.cache.retain(|_, (_, expires_at)| now < *expires_at);
    }
}

#[async_trait]
impl Actor for PermissionActor {
    fn name(&self) -> &str {
        "PermissionActor"
    }

    async fn handle_event(&mut self, _ctx: &ActorContext, event: &CoreEvent) -> ActorResponse {
        match event {
            CoreEvent::V1(v1) => match v1 {
                CoreEventV1::User(user_event) => {
                    self.handle_user_event(user_event);
                }
                // System events might affect permissions
                CoreEventV1::System(_) => {
                    // Could handle permission-related system events
                }
                _ => {}
            },
        }

        // Periodic cleanup
        if self.cache.len() > 1000 {
            self.cleanup_expired();
        }

        ActorResponse::Ack
    }

    async fn on_start(&mut self, ctx: &ActorContext) {
        log::info!(
            "[PermissionActor:{}] Started for tenant {}",
            ctx.actor_id,
            self.tenant_id
        );
    }

    async fn on_stop(&mut self, ctx: &ActorContext) {
        let stats = self.stats();
        log::info!(
            "[PermissionActor:{}] Stopped. Stats: {} hits, {} misses, {:.1}% hit rate",
            ctx.actor_id,
            stats.cache_hits,
            stats.cache_misses,
            stats.hit_rate
        );
    }
}

/// Statistics for permission actor
#[derive(Debug, Clone)]
pub struct PermissionActorStats {
    pub tenant_id: Uuid,
    pub cache_size: usize,
    pub cache_hits: u64,
    pub cache_misses: u64,
    pub hit_rate: f64,
    pub invalidations: u64,
}

// ═══════════════════════════════════════════════════════════════════════════════
// Tests
// ═══════════════════════════════════════════════════════════════════════════════

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_permission_cache() {
        let tenant_id = Uuid::new_v4();
        let mut actor = PermissionActor::new(tenant_id);
        let user_id = Uuid::new_v4();

        // Cache miss
        assert!(actor.check_cached(user_id, "system:read").is_none());

        // Cache permission
        actor.cache_permission(user_id, "system:read", true);

        // Cache hit
        assert_eq!(actor.check_cached(user_id, "system:read"), Some(true));
    }

    #[test]
    fn test_invalidation() {
        let tenant_id = Uuid::new_v4();
        let mut actor = PermissionActor::new(tenant_id);
        let user_id = Uuid::new_v4();

        actor.cache_permission(user_id, "system:read", true);
        actor.cache_permission(user_id, "system:write", false);

        actor.invalidate_user(user_id);

        assert!(actor.check_cached(user_id, "system:read").is_none());
        assert!(actor.check_cached(user_id, "system:write").is_none());
    }

    #[test]
    fn test_stats() {
        let tenant_id = Uuid::new_v4();
        let mut actor = PermissionActor::new(tenant_id);
        let user_id = Uuid::new_v4();

        // Miss
        actor.check_cached(user_id, "test");

        // Add and hit
        actor.cache_permission(user_id, "test", true);
        actor.check_cached(user_id, "test");
        actor.check_cached(user_id, "test");

        let stats = actor.stats();
        assert_eq!(stats.cache_hits, 2);
        assert_eq!(stats.cache_misses, 1);
    }
}

// ═══════════════════════════════════════════════════════════════════════════════
// Permission Cache — LRU Cache with TTL for Permission Lookups
// ═══════════════════════════════════════════════════════════════════════════════
// Reduces database queries for permission checks by caching results.
// Key: (user_id, tenant_id, permission_code)
// TTL: 5 minutes default
// ═══════════════════════════════════════════════════════════════════════════════

use std::collections::HashMap;
use std::sync::Arc;
use std::time::{Duration, Instant};
use tokio::sync::RwLock;
use uuid::Uuid;

/// Cache entry with expiration
#[derive(Debug, Clone)]
struct CacheEntry {
    value: bool,
    expires_at: Instant,
}

impl CacheEntry {
    fn new(value: bool, ttl: Duration) -> Self {
        Self {
            value,
            expires_at: Instant::now() + ttl,
        }
    }

    fn is_expired(&self) -> bool {
        Instant::now() > self.expires_at
    }
}

/// Permission cache key
#[derive(Debug, Clone, Hash, Eq, PartialEq)]
pub struct PermissionCacheKey {
    pub user_id: Uuid,
    pub tenant_id: Uuid,
    pub permission: String,
}

impl PermissionCacheKey {
    pub fn new(user_id: Uuid, tenant_id: Uuid, permission: impl Into<String>) -> Self {
        Self {
            user_id,
            tenant_id,
            permission: permission.into(),
        }
    }
}

/// Permission cache with LRU eviction and TTL
pub struct PermissionCache {
    cache: Arc<RwLock<HashMap<PermissionCacheKey, CacheEntry>>>,
    ttl: Duration,
    max_size: usize,
    // Metrics
    hits: Arc<RwLock<u64>>,
    misses: Arc<RwLock<u64>>,
}

impl PermissionCache {
    /// Create a new permission cache
    /// - ttl_secs: Time-to-live in seconds (default 300 = 5 min)
    /// - max_size: Maximum cache entries (default 10000)
    pub fn new(ttl_secs: u64, max_size: usize) -> Self {
        Self {
            cache: Arc::new(RwLock::new(HashMap::new())),
            ttl: Duration::from_secs(ttl_secs),
            max_size,
            hits: Arc::new(RwLock::new(0)),
            misses: Arc::new(RwLock::new(0)),
        }
    }

    /// Create with default settings (5 min TTL, 10k entries)
    pub fn default() -> Self {
        Self::new(300, 10000)
    }

    /// Get permission from cache
    /// Returns None if not found or expired
    pub async fn get(&self, key: &PermissionCacheKey) -> Option<bool> {
        let cache = self.cache.read().await;

        if let Some(entry) = cache.get(key) {
            if !entry.is_expired() {
                *self.hits.write().await += 1;
                return Some(entry.value);
            }
        }

        *self.misses.write().await += 1;
        None
    }

    /// Set permission in cache
    pub async fn set(&self, key: PermissionCacheKey, has_permission: bool) {
        let mut cache = self.cache.write().await;

        // Evict if at capacity (simple LRU - remove oldest expired first)
        if cache.len() >= self.max_size {
            self.evict_expired(&mut cache);

            // If still at capacity, remove random entries
            if cache.len() >= self.max_size {
                let keys_to_remove: Vec<_> =
                    cache.keys().take(self.max_size / 10).cloned().collect();
                for k in keys_to_remove {
                    cache.remove(&k);
                }
            }
        }

        cache.insert(key, CacheEntry::new(has_permission, self.ttl));
    }

    /// Invalidate cache for a specific user
    pub async fn invalidate_user(&self, user_id: Uuid) {
        let mut cache = self.cache.write().await;
        cache.retain(|k, _| k.user_id != user_id);
    }

    /// Invalidate cache for a specific tenant
    pub async fn invalidate_tenant(&self, tenant_id: Uuid) {
        let mut cache = self.cache.write().await;
        cache.retain(|k, _| k.tenant_id != tenant_id);
    }

    /// Invalidate all cache entries
    pub async fn invalidate_all(&self) {
        let mut cache = self.cache.write().await;
        cache.clear();
    }

    /// Get cache statistics
    pub async fn stats(&self) -> CacheStats {
        let hits = *self.hits.read().await;
        let misses = *self.misses.read().await;
        let size = self.cache.read().await.len();

        CacheStats {
            hits,
            misses,
            size,
            hit_rate: if hits + misses > 0 {
                (hits as f64 / (hits + misses) as f64) * 100.0
            } else {
                0.0
            },
        }
    }

    /// Remove expired entries
    fn evict_expired(&self, cache: &mut HashMap<PermissionCacheKey, CacheEntry>) {
        cache.retain(|_, v| !v.is_expired());
    }
}

/// Cache statistics
#[derive(Debug, Clone)]
pub struct CacheStats {
    pub hits: u64,
    pub misses: u64,
    pub size: usize,
    pub hit_rate: f64,
}

// ═══════════════════════════════════════════════════════════════════════════════
// Tests
// ═══════════════════════════════════════════════════════════════════════════════

#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test]
    async fn test_cache_hit() {
        let cache = PermissionCache::new(60, 100);
        let key = PermissionCacheKey::new(Uuid::new_v4(), Uuid::new_v4(), "system:read");

        cache.set(key.clone(), true).await;

        let result = cache.get(&key).await;
        assert_eq!(result, Some(true));
    }

    #[tokio::test]
    async fn test_cache_miss() {
        let cache = PermissionCache::new(60, 100);
        let key = PermissionCacheKey::new(Uuid::new_v4(), Uuid::new_v4(), "system:read");

        let result = cache.get(&key).await;
        assert_eq!(result, None);
    }

    #[tokio::test]
    async fn test_invalidate_user() {
        let cache = PermissionCache::new(60, 100);
        let user_id = Uuid::new_v4();
        let tenant_id = Uuid::new_v4();

        let key = PermissionCacheKey::new(user_id, tenant_id, "system:read");
        cache.set(key.clone(), true).await;

        cache.invalidate_user(user_id).await;

        let result = cache.get(&key).await;
        assert_eq!(result, None);
    }

    #[tokio::test]
    async fn test_stats() {
        let cache = PermissionCache::new(60, 100);
        let key = PermissionCacheKey::new(Uuid::new_v4(), Uuid::new_v4(), "system:read");

        cache.set(key.clone(), true).await;
        cache.get(&key).await; // hit
        cache.get(&key).await; // hit
        cache
            .get(&PermissionCacheKey::new(
                Uuid::new_v4(),
                Uuid::new_v4(),
                "other",
            ))
            .await; // miss

        let stats = cache.stats().await;
        assert_eq!(stats.hits, 2);
        assert_eq!(stats.misses, 1);
    }
}

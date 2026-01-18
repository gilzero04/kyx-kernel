use serde::{Deserialize, Serialize};
use uuid::Uuid;

/// Rate limits for signal connections
/// Values should be provided by kyx-plan plugin, not hardcoded
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SignalRateLimits {
    pub messages_per_minute: u32,
    pub connections: u32,
    pub rooms: u32,
}

// Default values when no plan plugin is installed
impl Default for SignalRateLimits {
    fn default() -> Self {
        Self {
            messages_per_minute: 100, // Generous default
            connections: 10,
            rooms: 20,
        }
    }
}

impl SignalRateLimits {
    /// Create rate limits with specific values
    pub fn new(messages_per_minute: u32, connections: u32, rooms: u32) -> Self {
        Self {
            messages_per_minute,
            connections,
            rooms,
        }
    }

    /// Parse rate limits from features JSON (from kyx-plan plugin)
    /// Falls back to defaults if fields missing
    pub fn from_features(features: Option<&serde_json::Value>) -> Self {
        if let Some(f) = features {
            Self {
                messages_per_minute: f
                    .get("messages_per_minute")
                    .and_then(|v| v.as_u64())
                    .unwrap_or(100) as u32,
                connections: f
                    .get("max_connections")
                    .and_then(|v| v.as_u64())
                    .unwrap_or(10) as u32,
                rooms: f.get("max_rooms").and_then(|v| v.as_u64()).unwrap_or(20) as u32,
            }
        } else {
            Self::default()
        }
    }
}

/// Claims structure for Signal Ticket JWT
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SignalTicketClaims {
    /// User ID
    pub sub: Uuid,
    /// Tenant ID
    pub tenant_id: Uuid,
    /// Mapped signal permissions
    pub signal_permissions: Vec<String>,
    /// Allowed rooms ("*" = all)
    pub allowed_rooms: Vec<String>,
    /// Plan type (optional, for display only)
    pub plan: Option<String>,
    /// Rate limits
    pub rate_limits: SignalRateLimits,
    /// Unique ticket ID (for single-use tracking)
    pub jti: Uuid,
    /// Issuer
    pub iss: String,
    /// Audience
    pub aud: String,
    /// Expiration timestamp
    pub exp: i64,
    /// Issued at timestamp
    pub iat: i64,
    /// Token type
    pub token_type: String,
}

impl SignalTicketClaims {
    pub fn new(
        user_id: Uuid,
        tenant_id: Uuid,
        signal_permissions: Vec<String>,
        plan: Option<String>,
        rate_limits: SignalRateLimits,
        ttl_seconds: i64,
    ) -> Self {
        let now = chrono::Utc::now().timestamp();
        Self {
            sub: user_id,
            tenant_id,
            signal_permissions,
            allowed_rooms: vec!["*".to_string()],
            plan,
            rate_limits,
            jti: Uuid::new_v4(),
            iss: "kyx-kernel".to_string(),
            aud: "kyx-signal".to_string(),
            exp: now + ttl_seconds,
            iat: now,
            token_type: "signal_ticket".to_string(),
        }
    }
}

/// Permission mapping from kernel to signal
pub struct PermissionMapper;

impl PermissionMapper {
    /// Maps kernel permissions to signal permissions
    pub fn map(kernel_permissions: &[String]) -> Vec<String> {
        let mapping = [
            ("chat:read", "subscribe"),
            ("chat:send", "publish"),
            ("room:join", "join"),
            ("room:admin", "moderate"),
            ("room:create", "create_room"),
        ];

        kernel_permissions
            .iter()
            .filter_map(|p| {
                mapping
                    .iter()
                    .find(|(k, _)| p == k)
                    .map(|(_, s)| s.to_string())
            })
            .collect()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_permission_mapping() {
        let kernel_perms = vec![
            "chat:read".to_string(),
            "chat:send".to_string(),
            "user:read".to_string(), // Not signal-related
        ];
        let signal_perms = PermissionMapper::map(&kernel_perms);

        assert_eq!(signal_perms.len(), 2);
        assert!(signal_perms.contains(&"subscribe".to_string()));
        assert!(signal_perms.contains(&"publish".to_string()));
    }

    #[test]
    fn test_rate_limits_from_features() {
        // Test with JSON features (as kyx-plan plugin would provide)
        let features = serde_json::json!({
            "messages_per_minute": 200,
            "max_connections": 10,
            "max_rooms": 50
        });
        let limits = SignalRateLimits::from_features(Some(&features));

        assert_eq!(limits.messages_per_minute, 200);
        assert_eq!(limits.connections, 10);
        assert_eq!(limits.rooms, 50);
    }

    #[test]
    fn test_rate_limits_defaults() {
        // Test without features (no kyx-plan plugin)
        let limits = SignalRateLimits::from_features(None);

        assert_eq!(limits.messages_per_minute, 100);
        assert_eq!(limits.connections, 10);
        assert_eq!(limits.rooms, 20);
    }
}

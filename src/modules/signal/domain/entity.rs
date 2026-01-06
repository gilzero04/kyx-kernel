use serde::{Deserialize, Serialize};
use uuid::Uuid;

/// Rate limits for signal connections based on plan
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SignalRateLimits {
    pub messages_per_minute: u32,
    pub connections: u32,
    pub rooms: u32,
}

impl Default for SignalRateLimits {
    fn default() -> Self {
        Self {
            messages_per_minute: 60,
            connections: 3,
            rooms: 10,
        }
    }
}

impl SignalRateLimits {
    pub fn for_plan(plan: &str) -> Self {
        match plan {
            "free" => Self {
                messages_per_minute: 30,
                connections: 2,
                rooms: 5,
            },
            "pro" => Self {
                messages_per_minute: 200,
                connections: 10,
                rooms: 50,
            },
            "enterprise" => Self {
                messages_per_minute: 1000,
                connections: 100,
                rooms: 500,
            },
            _ => Self::default(),
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
    /// Plan type
    pub plan: String,
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
        plan: String,
        ttl_seconds: i64,
    ) -> Self {
        let now = chrono::Utc::now().timestamp();
        Self {
            sub: user_id,
            tenant_id,
            signal_permissions,
            allowed_rooms: vec!["*".to_string()],
            plan: plan.clone(),
            rate_limits: SignalRateLimits::for_plan(&plan),
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
    fn test_rate_limits_by_plan() {
        let free = SignalRateLimits::for_plan("free");
        let pro = SignalRateLimits::for_plan("pro");
        let enterprise = SignalRateLimits::for_plan("enterprise");

        assert_eq!(free.connections, 2);
        assert_eq!(pro.connections, 10);
        assert_eq!(enterprise.connections, 100);
    }
}

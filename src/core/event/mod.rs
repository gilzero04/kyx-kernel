// ═══════════════════════════════════════════════════════════════════════════════
// Core Event System — Versioned Event Schema
// ═══════════════════════════════════════════════════════════════════════════════
// This module defines the central event types for the Kyx Kernel.
// All events are versioned to support backward-compatible evolution.
// NOTE: This module is reserved for future use, hence #[allow(dead_code)]
// ═══════════════════════════════════════════════════════════════════════════════

#![allow(dead_code)]
#![allow(clippy::enum_variant_names)]

use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use uuid::Uuid;

// ═══════════════════════════════════════════════════════════════════════════════
// Top-Level Versioned Event Wrapper
// ═══════════════════════════════════════════════════════════════════════════════

/// CoreEvent is the top-level event wrapper with version support.
/// When introducing breaking changes, add a new version (V2, V3, etc.)
/// without modifying existing versions.
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(tag = "version")]
pub enum CoreEvent {
    #[serde(rename = "v1")]
    V1(CoreEventV1),
}

impl CoreEvent {
    /// Create a new V1 event
    pub fn v1(event: CoreEventV1) -> Self {
        CoreEvent::V1(event)
    }

    /// Get the event type as a string for routing
    pub fn event_type(&self) -> &'static str {
        match self {
            CoreEvent::V1(e) => e.event_type(),
        }
    }
}

// ═══════════════════════════════════════════════════════════════════════════════
// V1 Event Schema
// ═══════════════════════════════════════════════════════════════════════════════

/// CoreEventV1 contains all domain events for version 1
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(tag = "domain", content = "payload")]
pub enum CoreEventV1 {
    Tenant(TenantEvent),
    User(UserEvent),
    Plugin(PluginEvent),
    Theme(ThemeEvent),
    Cms(CmsEvent),
    Media(MediaEvent),
    System(SystemEvent),
}

impl CoreEventV1 {
    pub fn event_type(&self) -> &'static str {
        match self {
            CoreEventV1::Tenant(_) => "tenant",
            CoreEventV1::User(_) => "user",
            CoreEventV1::Plugin(_) => "plugin",
            CoreEventV1::Theme(_) => "theme",
            CoreEventV1::Cms(_) => "cms",
            CoreEventV1::Media(_) => "media",
            CoreEventV1::System(_) => "system",
        }
    }
}

// ═══════════════════════════════════════════════════════════════════════════════
// Domain Events
// ═══════════════════════════════════════════════════════════════════════════════

/// Tenant-related events
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(tag = "action")]
pub enum TenantEvent {
    Created {
        tenant_id: Uuid,
        name: String,
        slug: String,
        parent_id: Option<Uuid>,
    },
    Updated {
        tenant_id: Uuid,
        changes: serde_json::Value,
    },
    Deleted {
        tenant_id: Uuid,
    },
    BrandingUpdated {
        tenant_id: Uuid,
        context: String,
    },
}

/// User-related events
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(tag = "action")]
pub enum UserEvent {
    Created {
        user_id: Uuid,
        email: String,
        tenant_id: Uuid,
    },
    Updated {
        user_id: Uuid,
        changes: serde_json::Value,
    },
    Deleted {
        user_id: Uuid,
    },
    LoggedIn {
        user_id: Uuid,
        tenant_id: Uuid,
        timestamp: DateTime<Utc>,
    },
    LoggedOut {
        user_id: Uuid,
        session_id: String,
    },
    PasswordChanged {
        user_id: Uuid,
    },
    RoleAssigned {
        user_id: Uuid,
        role_id: Uuid,
        tenant_id: Uuid,
    },
}

/// Plugin-related events
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(tag = "action")]
pub enum PluginEvent {
    Installed { plugin_id: String, version: String },
    Uninstalled { plugin_id: String },
    Enabled { plugin_id: String, tenant_id: Uuid },
    Disabled { plugin_id: String, tenant_id: Uuid },
    ConfigUpdated { plugin_id: String, tenant_id: Uuid },
}

/// Theme-related events
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(tag = "action")]
pub enum ThemeEvent {
    Created {
        theme_id: Uuid,
        slug: String,
    },
    Updated {
        theme_id: Uuid,
    },
    Deleted {
        theme_id: Uuid,
    },
    Activated {
        theme_id: Uuid,
        tenant_id: Uuid,
        context: String,
    },
    SharedStatusChanged {
        theme_id: Uuid,
        is_shared: bool,
    },
}

/// CMS-related events
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(tag = "action")]
pub enum CmsEvent {
    PageCreated {
        page_id: Uuid,
        tenant_id: Uuid,
        slug: String,
    },
    PageUpdated {
        page_id: Uuid,
    },
    PageDeleted {
        page_id: Uuid,
    },
    PagePublished {
        page_id: Uuid,
    },
    PageUnpublished {
        page_id: Uuid,
    },
}

/// Media-related events
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(tag = "action")]
pub enum MediaEvent {
    Uploaded {
        asset_id: Uuid,
        tenant_id: Uuid,
        filename: String,
    },
    Deleted {
        asset_id: Uuid,
    },
    FolderCreated {
        folder_id: Uuid,
        tenant_id: Uuid,
        name: String,
    },
}

/// System-related events
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(tag = "action")]
pub enum SystemEvent {
    ConfigUpdated {
        key: String,
        tenant_id: Option<Uuid>,
    },
    CorsUpdated {
        origins_count: usize,
    },
    HealthCheck {
        status: String,
    },
    VersionCheck {
        current: String,
        latest: String,
    },
}

// ═══════════════════════════════════════════════════════════════════════════════
// Event Metadata
// ═══════════════════════════════════════════════════════════════════════════════

/// Metadata attached to every event for tracing and auditing
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct EventMetadata {
    pub event_id: Uuid,
    pub timestamp: DateTime<Utc>,
    pub source: EventSource,
    pub correlation_id: Option<String>,
    pub actor: Option<EventActor>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum EventSource {
    Http,
    WebSocket,
    Internal,
    Scheduled,
    Plugin(String),
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct EventActor {
    pub user_id: Option<Uuid>,
    pub tenant_id: Option<Uuid>,
    pub ip_address: Option<String>,
}

impl EventMetadata {
    pub fn new(source: EventSource) -> Self {
        Self {
            event_id: Uuid::new_v4(),
            timestamp: Utc::now(),
            source,
            correlation_id: None,
            actor: None,
        }
    }

    pub fn with_actor(mut self, user_id: Option<Uuid>, tenant_id: Option<Uuid>) -> Self {
        self.actor = Some(EventActor {
            user_id,
            tenant_id,
            ip_address: None,
        });
        self
    }

    pub fn with_correlation(mut self, correlation_id: String) -> Self {
        self.correlation_id = Some(correlation_id);
        self
    }
}

// ═══════════════════════════════════════════════════════════════════════════════
// Event Envelope (Event + Metadata)
// ═══════════════════════════════════════════════════════════════════════════════

/// EventEnvelope wraps an event with its metadata
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct EventEnvelope {
    pub metadata: EventMetadata,
    pub event: CoreEvent,
}

impl EventEnvelope {
    pub fn new(event: CoreEvent, source: EventSource) -> Self {
        Self {
            metadata: EventMetadata::new(source),
            event,
        }
    }

    pub fn http(event: CoreEventV1) -> Self {
        Self::new(CoreEvent::v1(event), EventSource::Http)
    }

    pub fn internal(event: CoreEventV1) -> Self {
        Self::new(CoreEvent::v1(event), EventSource::Internal)
    }
}

// ═══════════════════════════════════════════════════════════════════════════════
// Tests
// ═══════════════════════════════════════════════════════════════════════════════

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_event_serialization() {
        let event = CoreEvent::v1(CoreEventV1::Tenant(TenantEvent::Created {
            tenant_id: Uuid::new_v4(),
            name: "Test Tenant".to_string(),
            slug: "test-tenant".to_string(),
            parent_id: None,
        }));

        let json = serde_json::to_string(&event).unwrap();
        assert!(json.contains("v1"));
        assert!(json.contains("tenant"));
        assert!(json.contains("Created"));
    }

    #[test]
    fn test_event_envelope() {
        let event = CoreEventV1::User(UserEvent::LoggedIn {
            user_id: Uuid::new_v4(),
            tenant_id: Uuid::new_v4(),
            timestamp: Utc::now(),
        });

        let envelope = EventEnvelope::http(event);
        assert!(matches!(envelope.metadata.source, EventSource::Http));
    }

    #[test]
    fn test_event_type_routing() {
        let event = CoreEvent::v1(CoreEventV1::Plugin(PluginEvent::Installed {
            plugin_id: "my-plugin".to_string(),
            version: "1.0.0".to_string(),
        }));

        assert_eq!(event.event_type(), "plugin");
    }
}

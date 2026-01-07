// ════════════════════════════════════════════════════════════════════════════
// Notification Client - Thin HTTP Client for kyx-signal
// ════════════════════════════════════════════════════════════════════════════

use std::sync::Arc;
use ntex::http::client::Client;
use serde::{Deserialize, Serialize};
use uuid::Uuid;
use log::{info, warn, error};
use tokio::sync::RwLock;
use std::collections::VecDeque;

/// Notification types
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum NotificationType {
    System,
    User,
    Payment,
    Subscription,
    Alert,
    Custom(String),
}

/// Notification priority
#[derive(Debug, Clone, Copy, Serialize, Deserialize, Default)]
#[serde(rename_all = "lowercase")]
pub enum NotificationPriority {
    Low,
    #[default]
    Normal,
    High,
    Critical,
}

/// Notification payload to send to kyx-signal
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Notification {
    /// Target user ID
    pub user_id: Uuid,
    /// Tenant ID
    pub tenant_id: Uuid,
    /// Notification type
    pub notification_type: NotificationType,
    /// Priority
    #[serde(default)]
    pub priority: NotificationPriority,
    /// Title
    pub title: String,
    /// Body message
    pub body: String,
    /// Optional data payload
    #[serde(skip_serializing_if = "Option::is_none")]
    pub data: Option<serde_json::Value>,
    /// Channels to deliver on (websocket, push, email)
    #[serde(default)]
    pub channels: Vec<String>,
}

/// Notification response from kyx-signal
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct NotificationResponse {
    pub success: bool,
    pub message_id: Option<String>,
    pub delivered_channels: Vec<String>,
}

/// Queue entry for retry
#[derive(Debug, Clone)]
struct QueuedNotification {
    notification: Notification,
    retries: u8,
    #[allow(dead_code)]
    created_at: std::time::Instant,
}

/// Notification client for sending notifications via kyx-signal
pub struct NotificationClient {
    /// Base URL of kyx-signal service
    signal_url: String,
    /// Retry queue for failed notifications
    retry_queue: Arc<RwLock<VecDeque<QueuedNotification>>>,
    /// Max retries
    max_retries: u8,
    /// Whether client is available (signal service reachable)
    available: Arc<RwLock<bool>>,
}

impl NotificationClient {
    /// Create a new notification client
    pub fn new() -> Self {
        let signal_url = std::env::var("SIGNAL_API_URL")
            .unwrap_or_else(|_| "http://localhost:8081".to_string());
        
        Self {
            signal_url,
            retry_queue: Arc::new(RwLock::new(VecDeque::new())),
            max_retries: 3,
            available: Arc::new(RwLock::new(true)),
        }
    }
    
    /// Create with custom URL
    pub fn with_url(url: impl Into<String>) -> Self {
        let mut client = Self::new();
        client.signal_url = url.into();
        client
    }
    
    /// Check if notification service is available
    pub async fn is_available(&self) -> bool {
        *self.available.read().await
    }
    
    /// Send a notification to a user using ntex HTTP client
    pub async fn notify(&self, notification: Notification) -> Result<NotificationResponse, NotificationError> {
        let url = format!("{}/api/v1/notify", self.signal_url);
        
        // Use ntex HTTP client (same pattern as ai_service.rs)
        let client = Client::build()
            .timeout(std::time::Duration::from_secs(10))
            .finish();
        
        match client
            .post(&url)
            .header("Content-Type", "application/json")
            .send_json(&notification)
            .await
        {
            Ok(mut response) => {
                if response.status().is_success() {
                    match response.json::<NotificationResponse>().await {
                        Ok(resp) => {
                            info!(
                                "Notification sent to user {} via {:?}",
                                notification.user_id,
                                resp.delivered_channels
                            );
                            
                            // Mark as available
                            *self.available.write().await = true;
                            
                            Ok(resp)
                        }
                        Err(e) => {
                            Err(NotificationError::ParseError(e.to_string()))
                        }
                    }
                } else {
                    let status = response.status().as_u16();
                    warn!("Notification failed with status {}", status);
                    
                    // Queue for retry if server error
                    if status >= 500 {
                        self.queue_for_retry(notification).await;
                    }
                    
                    Err(NotificationError::ServerError(status, "Server error".to_string()))
                }
            }
            Err(e) => {
                error!("Failed to send notification: {}", e);
                
                // Mark as unavailable and queue for retry
                *self.available.write().await = false;
                self.queue_for_retry(notification).await;
                
                Err(NotificationError::ConnectionError(e.to_string()))
            }
        }
    }
    
    /// Broadcast notification to all users in a tenant
    pub async fn broadcast(&self, tenant_id: Uuid, title: &str, body: &str) -> Result<(), NotificationError> {
        let url = format!("{}/api/v1/broadcast", self.signal_url);
        
        let payload = serde_json::json!({
            "tenant_id": tenant_id,
            "title": title,
            "body": body,
            "notification_type": "system"
        });
        
        let client = Client::build()
            .timeout(std::time::Duration::from_secs(10))
            .finish();
        
        match client
            .post(&url)
            .header("Content-Type", "application/json")
            .send_json(&payload)
            .await
        {
            Ok(response) if response.status().is_success() => {
                info!("Broadcast sent to tenant {}", tenant_id);
                Ok(())
            }
            Ok(response) => {
                Err(NotificationError::ServerError(
                    response.status().as_u16(),
                    "Broadcast failed".to_string()
                ))
            }
            Err(e) => {
                Err(NotificationError::ConnectionError(e.to_string()))
            }
        }
    }
    
    /// Queue notification for retry
    async fn queue_for_retry(&self, notification: Notification) {
        let mut queue = self.retry_queue.write().await;
        
        // Limit queue size
        if queue.len() < 1000 {
            queue.push_back(QueuedNotification {
                notification,
                retries: 0,
                created_at: std::time::Instant::now(),
            });
        } else {
            warn!("Notification queue full, dropping notification");
        }
    }
    
    /// Process retry queue (call periodically)
    pub async fn process_retry_queue(&self) {
        let mut queue = self.retry_queue.write().await;
        let mut to_retry = Vec::new();
        
        // Collect items to retry
        while let Some(mut item) = queue.pop_front() {
            if item.retries < self.max_retries {
                item.retries += 1;
                to_retry.push(item);
            } else {
                warn!(
                    "Notification to user {} failed after {} retries",
                    item.notification.user_id,
                    self.max_retries
                );
            }
        }
        
        // Release lock before retrying
        drop(queue);
        
        // Retry notifications
        for item in to_retry {
            if let Err(e) = self.notify(item.notification.clone()).await {
                // Will be re-queued automatically
                warn!("Retry failed: {:?}", e);
            }
        }
    }
    
    /// Get queue size
    pub async fn queue_size(&self) -> usize {
        self.retry_queue.read().await.len()
    }
}

impl Default for NotificationClient {
    fn default() -> Self {
        Self::new()
    }
}

/// Notification errors
#[derive(Debug, Clone)]
pub enum NotificationError {
    ConnectionError(String),
    ServerError(u16, String),
    ParseError(String),
    #[allow(dead_code)]
    ServiceUnavailable,
}

impl std::fmt::Display for NotificationError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::ConnectionError(e) => write!(f, "Connection error: {}", e),
            Self::ServerError(code, msg) => write!(f, "Server error {}: {}", code, msg),
            Self::ParseError(e) => write!(f, "Parse error: {}", e),
            Self::ServiceUnavailable => write!(f, "Notification service unavailable"),
        }
    }
}

impl std::error::Error for NotificationError {}

// ════════════════════════════════════════════════════════════════════════════
// Helper constructors for common notification types
// ════════════════════════════════════════════════════════════════════════════

impl Notification {
    /// Create a system notification
    pub fn system(user_id: Uuid, tenant_id: Uuid, title: &str, body: &str) -> Self {
        Self {
            user_id,
            tenant_id,
            notification_type: NotificationType::System,
            priority: NotificationPriority::Normal,
            title: title.to_string(),
            body: body.to_string(),
            data: None,
            channels: vec!["websocket".to_string(), "push".to_string()],
        }
    }
    
    /// Create a payment notification
    pub fn payment(user_id: Uuid, tenant_id: Uuid, amount: &str, currency: &str) -> Self {
        Self {
            user_id,
            tenant_id,
            notification_type: NotificationType::Payment,
            priority: NotificationPriority::High,
            title: "Payment Received".to_string(),
            body: format!("You received {} {}", amount, currency),
            data: Some(serde_json::json!({
                "amount": amount,
                "currency": currency
            })),
            channels: vec!["websocket".to_string(), "push".to_string()],
        }
    }
    
    /// Create an alert notification
    pub fn alert(user_id: Uuid, tenant_id: Uuid, title: &str, body: &str) -> Self {
        Self {
            user_id,
            tenant_id,
            notification_type: NotificationType::Alert,
            priority: NotificationPriority::Critical,
            title: title.to_string(),
            body: body.to_string(),
            data: None,
            channels: vec!["websocket".to_string(), "push".to_string(), "email".to_string()],
        }
    }
    
    /// Add data payload
    #[allow(dead_code)]
    pub fn with_data(mut self, data: serde_json::Value) -> Self {
        self.data = Some(data);
        self
    }
    
    /// Set channels
    #[allow(dead_code)]
    pub fn with_channels(mut self, channels: Vec<String>) -> Self {
        self.channels = channels;
        self
    }
}

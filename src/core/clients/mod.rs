// Core HTTP clients for external services
pub mod notification_client;

pub use notification_client::{NotificationClient, Notification, NotificationError, NotificationType, NotificationPriority};

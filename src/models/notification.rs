use chrono::Utc;
use serde::{Deserialize, Serialize};
use sqlx::FromRow;
use uuid::Uuid;
use validator::Validate;

#[derive(Debug, Clone, Serialize, Deserialize, FromRow)]
pub struct Notification {
    pub id: String,
    pub user_id: String,
    pub title: String,
    pub message: String,
    pub notification_type: String, // renamed from 'type' to avoid keyword conflict
    pub entity_type: Option<String>,
    pub entity_id: Option<String>,
    pub read_at: Option<String>,
    pub created_at: String,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub enum NotificationType {
    Info,
    Success,
    Warning,
    Error,
}

impl From<String> for NotificationType {
    fn from(s: String) -> Self {
        match s.as_str() {
            "info" => NotificationType::Info,
            "success" => NotificationType::Success,
            "warning" => NotificationType::Warning,
            "error" => NotificationType::Error,
            _ => NotificationType::Info,
        }
    }
}

impl From<NotificationType> for String {
    fn from(nt: NotificationType) -> Self {
        match nt {
            NotificationType::Info => "info".to_string(),
            NotificationType::Success => "success".to_string(),
            NotificationType::Warning => "warning".to_string(),
            NotificationType::Error => "error".to_string(),
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize, FromRow)]
pub struct Activity {
    pub id: String,
    pub user_id: String,
    pub action: String,
    pub entity_type: String,
    pub entity_id: String,
    pub details: Option<String>, // JSON string
    pub ip_address: Option<String>,
    pub user_agent: Option<String>,
    pub created_at: String,
}

#[derive(Debug, Deserialize, Validate)]
pub struct CreateNotificationRequest {
    #[validate(length(min = 1, max = 255))]
    pub title: String,
    
    #[validate(length(min = 1, max = 1000))]
    pub message: String,
    
    pub notification_type: Option<NotificationType>,
    pub entity_type: Option<String>,
    pub entity_id: Option<String>,
}

#[derive(Debug, Serialize)]
pub struct NotificationResponse {
    pub id: String,
    pub title: String,
    pub message: String,
    pub notification_type: NotificationType,
    pub entity_type: Option<String>,
    pub entity_id: Option<String>,
    pub read_at: Option<String>,
    pub created_at: String,
}

impl From<Notification> for NotificationResponse {
    fn from(notification: Notification) -> Self {
        Self {
            id: notification.id,
            title: notification.title,
            message: notification.message,
            notification_type: NotificationType::from(notification.notification_type),
            entity_type: notification.entity_type,
            entity_id: notification.entity_id,
            read_at: notification.read_at,
            created_at: notification.created_at,
        }
    }
}

#[derive(Debug, Serialize)]
pub struct ActivityResponse {
    pub id: String,
    pub user_id: String,
    pub username: Option<String>,
    pub action: String,
    pub entity_type: String,
    pub entity_id: String,
    pub details: Option<serde_json::Value>,
    pub ip_address: Option<String>,
    pub user_agent: Option<String>,
    pub created_at: String,
}

impl Notification {
    pub fn new(
        user_id: String,
        title: String,
        message: String,
        notification_type: NotificationType,
        entity_type: Option<String>,
        entity_id: Option<String>,
    ) -> Self {
        Self {
            id: Uuid::new_v4().to_string(),
            user_id,
            title,
            message,
            notification_type: String::from(notification_type),
            entity_type,
            entity_id,
            read_at: None,
            created_at: Utc::now().to_rfc3339(),
        }
    }
}

impl Activity {
    pub fn new(
        user_id: String,
        action: String,
        entity_type: String,
        entity_id: String,
        details: Option<serde_json::Value>,
        ip_address: Option<String>,
        user_agent: Option<String>,
    ) -> Self {
        let details_json = details.map(|d| serde_json::to_string(&d).unwrap_or_default());
        
        Self {
            id: Uuid::new_v4().to_string(),
            user_id,
            action,
            entity_type,
            entity_id,
            details: details_json,
            ip_address,
            user_agent,
            created_at: Utc::now().to_rfc3339(),
        }
    }
}
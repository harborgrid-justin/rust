use anyhow::Result;
use sqlx::SqlitePool;
use tracing::info;

use crate::models::{
    Notification, Activity, NotificationType, CreateNotificationRequest,
    NotificationResponse, ActivityResponse
};

pub async fn create_notification(
    db: &SqlitePool,
    user_id: String,
    request: CreateNotificationRequest,
) -> Result<Notification> {
    let notification = Notification::new(
        user_id,
        request.title,
        request.message,
        request.notification_type.unwrap_or(NotificationType::Info),
        request.entity_type,
        request.entity_id,
    );

    sqlx::query!(
        r#"
        INSERT INTO notifications (id, user_id, title, message, notification_type, entity_type, entity_id, read_at, created_at)
        VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7, ?8, ?9)
        "#,
        notification.id,
        notification.user_id,
        notification.title,
        notification.message,
        notification.notification_type,
        notification.entity_type,
        notification.entity_id,
        notification.read_at,
        notification.created_at
    )
    .execute(db)
    .await?;

    info!("Created notification for user {}", notification.user_id);
    Ok(notification)
}

pub async fn get_user_notifications(
    db: &SqlitePool,
    user_id: &str,
    unread_only: bool,
    limit: i32,
    offset: i32,
) -> Result<Vec<NotificationResponse>> {
    let rows = if unread_only {
        sqlx::query!(
            r#"
            SELECT id, user_id, title, message, notification_type, entity_type, entity_id, read_at, created_at
            FROM notifications
            WHERE user_id = ?1 AND read_at IS NULL
            ORDER BY created_at DESC
            LIMIT ?2 OFFSET ?3
            "#,
            user_id,
            limit,
            offset
        )
        .fetch_all(db)
        .await?
    } else {
        sqlx::query!(
            r#"
            SELECT id, user_id, title, message, notification_type, entity_type, entity_id, read_at, created_at
            FROM notifications
            WHERE user_id = ?1
            ORDER BY created_at DESC
            LIMIT ?2 OFFSET ?3
            "#,
            user_id,
            limit,
            offset
        )
        .fetch_all(db)
        .await?
    };

    Ok(rows
        .into_iter()
        .map(|row| {
            let notification = Notification {
                id: row.id.unwrap_or_default(),
                user_id: row.user_id.unwrap_or_default(),
                title: row.title.unwrap_or_default(),
                message: row.message.unwrap_or_default(),
                notification_type: row.notification_type.unwrap_or_default(),
                entity_type: row.entity_type,
                entity_id: row.entity_id,
                read_at: row.read_at,
                created_at: row.created_at.unwrap_or_default(),
            };
            NotificationResponse::from(notification)
        })
        .collect())
}

pub async fn mark_notification_read(
    db: &SqlitePool,
    notification_id: &str,
    user_id: &str,
) -> Result<()> {
    let now = chrono::Utc::now().to_rfc3339();
    
    sqlx::query!(
        "UPDATE notifications SET read_at = ?1 WHERE id = ?2 AND user_id = ?3",
        now,
        notification_id,
        user_id
    )
    .execute(db)
    .await?;

    info!("Marked notification {} as read for user {}", notification_id, user_id);
    Ok(())
}

pub async fn mark_all_notifications_read(db: &SqlitePool, user_id: &str) -> Result<()> {
    let now = chrono::Utc::now().to_rfc3339();
    
    sqlx::query!(
        "UPDATE notifications SET read_at = ?1 WHERE user_id = ?2 AND read_at IS NULL",
        now,
        user_id
    )
    .execute(db)
    .await?;

    info!("Marked all notifications as read for user {}", user_id);
    Ok(())
}

pub async fn get_notification_count(
    db: &SqlitePool,
    user_id: &str,
    unread_only: bool,
) -> Result<i32> {
    let count = if unread_only {
        sqlx::query!(
            "SELECT COUNT(*) as count FROM notifications WHERE user_id = ?1 AND read_at IS NULL",
            user_id
        )
        .fetch_one(db)
        .await?
        .count
    } else {
        sqlx::query!(
            "SELECT COUNT(*) as count FROM notifications WHERE user_id = ?1",
            user_id
        )
        .fetch_one(db)
        .await?
        .count
    };

    Ok(count.unwrap_or(0) as i32)
}

pub async fn log_activity(
    db: &SqlitePool,
    user_id: String,
    action: String,
    entity_type: String,
    entity_id: String,
    details: Option<serde_json::Value>,
    ip_address: Option<String>,
    user_agent: Option<String>,
) -> Result<()> {
    let activity = Activity::new(
        user_id,
        action,
        entity_type,
        entity_id,
        details,
        ip_address,
        user_agent,
    );

    sqlx::query!(
        r#"
        INSERT INTO activities (id, user_id, action, entity_type, entity_id, details, ip_address, user_agent, created_at)
        VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7, ?8, ?9)
        "#,
        activity.id,
        activity.user_id,
        activity.action,
        activity.entity_type,
        activity.entity_id,
        activity.details,
        activity.ip_address,
        activity.user_agent,
        activity.created_at
    )
    .execute(db)
    .await?;

    Ok(())
}

pub async fn get_user_activities(
    db: &SqlitePool,
    user_id: &str,
    entity_type: Option<&str>,
    limit: i32,
    offset: i32,
) -> Result<Vec<ActivityResponse>> {
    let rows = match entity_type {
        Some(et) => {
            sqlx::query!(
                r#"
                SELECT a.id, a.user_id, a.action, a.entity_type, a.entity_id, a.details, 
                       a.ip_address, a.user_agent, a.created_at, u.username
                FROM activities a
                INNER JOIN users u ON a.user_id = u.id
                WHERE a.user_id = ?1 AND a.entity_type = ?2
                ORDER BY a.created_at DESC
                LIMIT ?3 OFFSET ?4
                "#,
                user_id,
                et,
                limit,
                offset
            )
            .fetch_all(db)
            .await?
        }
        None => {
            sqlx::query!(
                r#"
                SELECT a.id, a.user_id, a.action, a.entity_type, a.entity_id, a.details, 
                       a.ip_address, a.user_agent, a.created_at, u.username
                FROM activities a
                INNER JOIN users u ON a.user_id = u.id
                WHERE a.user_id = ?1
                ORDER BY a.created_at DESC
                LIMIT ?2 OFFSET ?3
                "#,
                user_id,
                limit,
                offset
            )
            .fetch_all(db)
            .await?
        }
    };

    Ok(rows
        .into_iter()
        .map(|row| {
            let details = row.details
                .as_ref()
                .and_then(|d| serde_json::from_str::<serde_json::Value>(d).ok());

            ActivityResponse {
                id: row.id.unwrap_or_default(),
                user_id: row.user_id.unwrap_or_default(),
                username: row.username,
                action: row.action.unwrap_or_default(),
                entity_type: row.entity_type.unwrap_or_default(),
                entity_id: row.entity_id.unwrap_or_default(),
                details,
                ip_address: row.ip_address,
                user_agent: row.user_agent,
                created_at: row.created_at.unwrap_or_default(),
            }
        })
        .collect())
}

pub async fn get_system_activities(
    db: &SqlitePool,
    entity_type: Option<&str>,
    limit: i32,
    offset: i32,
) -> Result<Vec<ActivityResponse>> {
    let rows = match entity_type {
        Some(et) => {
            sqlx::query!(
                r#"
                SELECT a.id, a.user_id, a.action, a.entity_type, a.entity_id, a.details, 
                       a.ip_address, a.user_agent, a.created_at, u.username
                FROM activities a
                INNER JOIN users u ON a.user_id = u.id
                WHERE a.entity_type = ?1
                ORDER BY a.created_at DESC
                LIMIT ?2 OFFSET ?3
                "#,
                et,
                limit,
                offset
            )
            .fetch_all(db)
            .await?
        }
        None => {
            sqlx::query!(
                r#"
                SELECT a.id, a.user_id, a.action, a.entity_type, a.entity_id, a.details, 
                       a.ip_address, a.user_agent, a.created_at, u.username
                FROM activities a
                INNER JOIN users u ON a.user_id = u.id
                ORDER BY a.created_at DESC
                LIMIT ?1 OFFSET ?2
                "#,
                limit,
                offset
            )
            .fetch_all(db)
            .await?
        }
    };

    Ok(rows
        .into_iter()
        .map(|row| {
            let details = row.details
                .as_ref()
                .and_then(|d| serde_json::from_str::<serde_json::Value>(d).ok());

            ActivityResponse {
                id: row.id.unwrap_or_default(),
                user_id: row.user_id.unwrap_or_default(),
                username: row.username,
                action: row.action.unwrap_or_default(),
                entity_type: row.entity_type.unwrap_or_default(),
                entity_id: row.entity_id.unwrap_or_default(),
                details,
                ip_address: row.ip_address,
                user_agent: row.user_agent,
                created_at: row.created_at.unwrap_or_default(),
            }
        })
        .collect())
}
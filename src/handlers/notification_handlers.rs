use axum::{
    extract::{Path, Query, State},
    http::StatusCode,
    response::Json,
};
use serde_json::{json, Value};
use sqlx::SqlitePool;
use std::collections::HashMap;
use tracing::error;
use validator::Validate;

use crate::models::CreateNotificationRequest;
use crate::services::{notification_service};

pub async fn get_notifications(
    State(db): State<SqlitePool>,
    Query(params): Query<HashMap<String, String>>,
) -> Result<Json<Value>, StatusCode> {
    // TODO: Extract user ID from JWT token
    let user_id = "system"; // Temporary placeholder

    let unread_only = params.get("unread").map(|u| u == "true").unwrap_or(false);
    let limit = params
        .get("limit")
        .and_then(|l| l.parse().ok())
        .unwrap_or(20);
    let offset = params
        .get("offset")
        .and_then(|o| o.parse().ok())
        .unwrap_or(0);

    match notification_service::get_user_notifications(&db, user_id, unread_only, limit, offset).await {
        Ok(notifications) => Ok(Json(json!({
            "notifications": notifications,
            "count": notifications.len()
        }))),
        Err(err) => {
            error!("Failed to get notifications: {:?}", err);
            Err(StatusCode::INTERNAL_SERVER_ERROR)
        }
    }
}

pub async fn create_notification(
    State(db): State<SqlitePool>,
    Json(payload): Json<CreateNotificationRequest>,
) -> Result<Json<Value>, StatusCode> {
    if let Err(err) = payload.validate() {
        error!("Notification validation failed: {:?}", err);
        return Err(StatusCode::BAD_REQUEST);
    }

    // TODO: Extract user ID from JWT token
    let user_id = "system".to_string(); // Temporary placeholder

    match notification_service::create_notification(&db, user_id, payload).await {
        Ok(notification) => Ok(Json(json!({
            "message": "Notification created successfully",
            "notification_id": notification.id
        }))),
        Err(err) => {
            error!("Failed to create notification: {:?}", err);
            Err(StatusCode::INTERNAL_SERVER_ERROR)
        }
    }
}

pub async fn mark_notification_read(
    State(db): State<SqlitePool>,
    Path(notification_id): Path<String>,
) -> Result<Json<Value>, StatusCode> {
    // TODO: Extract user ID from JWT token
    let user_id = "system"; // Temporary placeholder

    match notification_service::mark_notification_read(&db, &notification_id, user_id).await {
        Ok(_) => Ok(Json(json!({
            "message": "Notification marked as read",
            "notification_id": notification_id
        }))),
        Err(err) => {
            error!("Failed to mark notification as read: {:?}", err);
            Err(StatusCode::INTERNAL_SERVER_ERROR)
        }
    }
}

pub async fn mark_all_notifications_read(
    State(db): State<SqlitePool>,
) -> Result<Json<Value>, StatusCode> {
    // TODO: Extract user ID from JWT token
    let user_id = "system"; // Temporary placeholder

    match notification_service::mark_all_notifications_read(&db, user_id).await {
        Ok(_) => Ok(Json(json!({
            "message": "All notifications marked as read"
        }))),
        Err(err) => {
            error!("Failed to mark all notifications as read: {:?}", err);
            Err(StatusCode::INTERNAL_SERVER_ERROR)
        }
    }
}

pub async fn get_notification_count(
    State(db): State<SqlitePool>,
    Query(params): Query<HashMap<String, String>>,
) -> Result<Json<Value>, StatusCode> {
    // TODO: Extract user ID from JWT token
    let user_id = "system"; // Temporary placeholder

    let unread_only = params.get("unread").map(|u| u == "true").unwrap_or(true);

    match notification_service::get_notification_count(&db, user_id, unread_only).await {
        Ok(count) => Ok(Json(json!({
            "count": count,
            "unread_only": unread_only
        }))),
        Err(err) => {
            error!("Failed to get notification count: {:?}", err);
            Err(StatusCode::INTERNAL_SERVER_ERROR)
        }
    }
}

pub async fn get_user_activities(
    State(db): State<SqlitePool>,
    Query(params): Query<HashMap<String, String>>,
) -> Result<Json<Value>, StatusCode> {
    // TODO: Extract user ID from JWT token
    let user_id = "system"; // Temporary placeholder

    let entity_type = params.get("entity_type").map(|e| e.as_str());
    let limit = params
        .get("limit")
        .and_then(|l| l.parse().ok())
        .unwrap_or(20);
    let offset = params
        .get("offset")
        .and_then(|o| o.parse().ok())
        .unwrap_or(0);

    match notification_service::get_user_activities(&db, user_id, entity_type, limit, offset).await {
        Ok(activities) => Ok(Json(json!({
            "activities": activities,
            "count": activities.len()
        }))),
        Err(err) => {
            error!("Failed to get user activities: {:?}", err);
            Err(StatusCode::INTERNAL_SERVER_ERROR)
        }
    }
}

pub async fn get_system_activities(
    State(db): State<SqlitePool>,
    Query(params): Query<HashMap<String, String>>,
) -> Result<Json<Value>, StatusCode> {
    let entity_type = params.get("entity_type").map(|e| e.as_str());
    let limit = params
        .get("limit")
        .and_then(|l| l.parse().ok())
        .unwrap_or(20);
    let offset = params
        .get("offset")
        .and_then(|o| o.parse().ok())
        .unwrap_or(0);

    match notification_service::get_system_activities(&db, entity_type, limit, offset).await {
        Ok(activities) => Ok(Json(json!({
            "activities": activities,
            "count": activities.len()
        }))),
        Err(err) => {
            error!("Failed to get system activities: {:?}", err);
            Err(StatusCode::INTERNAL_SERVER_ERROR)
        }
    }
}
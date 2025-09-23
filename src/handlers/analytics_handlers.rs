use axum::{
    extract::{Query, State},
    http::StatusCode,
    response::Json,
};
use serde_json::{json, Value};
use sqlx::SqlitePool;
use std::collections::HashMap;
use tracing::error;

pub async fn get_dashboard_stats(
    State(db): State<SqlitePool>,
) -> Result<Json<Value>, StatusCode> {
    // TODO: Extract user ID from JWT token for user-specific stats
    
    // Get total counts
    let user_count = sqlx::query!("SELECT COUNT(*) as count FROM users")
        .fetch_one(&db)
        .await
        .map(|r| r.count.unwrap_or(0))
        .unwrap_or(0);

    let document_count = sqlx::query!("SELECT COUNT(*) as count FROM documents")
        .fetch_one(&db)
        .await
        .map(|r| r.count.unwrap_or(0))
        .unwrap_or(0);

    let case_count = sqlx::query!("SELECT COUNT(*) as count FROM cases")
        .fetch_one(&db)
        .await
        .map(|r| r.count.unwrap_or(0))
        .unwrap_or(0);

    let team_count = sqlx::query!("SELECT COUNT(*) as count FROM teams")
        .fetch_one(&db)
        .await
        .map(|r| r.count.unwrap_or(0))
        .unwrap_or(0);

    // Get active cases by status
    let open_cases = sqlx::query!("SELECT COUNT(*) as count FROM cases WHERE status = 'open'")
        .fetch_one(&db)
        .await
        .map(|r| r.count.unwrap_or(0))
        .unwrap_or(0);

    let in_progress_cases = sqlx::query!("SELECT COUNT(*) as count FROM cases WHERE status = 'inprogress'")
        .fetch_one(&db)
        .await
        .map(|r| r.count.unwrap_or(0))
        .unwrap_or(0);

    // Get workflow metrics
    let completed_workflows = sqlx::query!("SELECT COUNT(*) as count FROM case_workflows WHERE status = 'completed'")
        .fetch_one(&db)
        .await
        .map(|r| r.count.unwrap_or(0))
        .unwrap_or(0);

    let pending_workflows = sqlx::query!("SELECT COUNT(*) as count FROM case_workflows WHERE status = 'pending'")
        .fetch_one(&db)
        .await
        .map(|r| r.count.unwrap_or(0))
        .unwrap_or(0);

    // Get recent activity (last 7 days)
    let recent_cases = sqlx::query!(
        "SELECT COUNT(*) as count FROM cases WHERE created_at >= datetime('now', '-7 days')"
    )
    .fetch_one(&db)
    .await
    .map(|r| r.count.unwrap_or(0))
    .unwrap_or(0);

    let recent_documents = sqlx::query!(
        "SELECT COUNT(*) as count FROM documents WHERE created_at >= datetime('now', '-7 days')"
    )
    .fetch_one(&db)
    .await
    .map(|r| r.count.unwrap_or(0))
    .unwrap_or(0);

    // Calculate productivity metrics
    let avg_case_resolution_time = calculate_avg_resolution_time(&db).await.unwrap_or(0.0);
    let workflow_efficiency = if pending_workflows + completed_workflows > 0 {
        (completed_workflows as f64 / (pending_workflows + completed_workflows) as f64) * 100.0
    } else {
        0.0
    };

    Ok(Json(json!({
        "totals": {
            "users": user_count,
            "documents": document_count,
            "cases": case_count,
            "teams": team_count
        },
        "case_status_breakdown": {
            "open": open_cases,
            "in_progress": in_progress_cases,
            "total_active": open_cases + in_progress_cases
        },
        "workflow_metrics": {
            "completed_workflows": completed_workflows,
            "pending_workflows": pending_workflows,
            "efficiency_percentage": workflow_efficiency.round()
        },
        "recent_activity": {
            "cases_last_7_days": recent_cases,
            "documents_last_7_days": recent_documents
        },
        "performance_indicators": {
            "avg_case_resolution_hours": avg_case_resolution_time,
            "workflow_completion_rate": workflow_efficiency
        },
        "timestamp": chrono::Utc::now().to_rfc3339()
    })))
}

pub async fn get_case_analytics(
    State(db): State<SqlitePool>,
    Query(params): Query<HashMap<String, String>>,
) -> Result<Json<Value>, StatusCode> {
    let days = params
        .get("days")
        .and_then(|d| d.parse().ok())
        .unwrap_or(30);

    // Cases created in the last N days
    let cases_by_day = sqlx::query!(
        r#"
        SELECT DATE(created_at) as date, COUNT(*) as count
        FROM cases 
        WHERE created_at >= datetime('now', '-' || ? || ' days')
        GROUP BY DATE(created_at)
        ORDER BY date
        "#,
        days
    )
    .fetch_all(&db)
    .await
    .unwrap_or_default();

    // Cases by priority
    let cases_by_priority = sqlx::query!(
        r#"
        SELECT priority, COUNT(*) as count
        FROM cases
        GROUP BY priority
        "#
    )
    .fetch_all(&db)
    .await
    .unwrap_or_default();

    // Cases by status
    let cases_by_status = sqlx::query!(
        r#"
        SELECT status, COUNT(*) as count
        FROM cases
        GROUP BY status
        "#
    )
    .fetch_all(&db)
    .await
    .unwrap_or_default();

    Ok(Json(json!({
        "period_days": days,
        "cases_by_day": cases_by_day.into_iter().map(|r| json!({
            "date": r.date.unwrap_or_default(),
            "count": r.count.unwrap_or(0)
        })).collect::<Vec<_>>(),
        "cases_by_priority": cases_by_priority.into_iter().map(|r| json!({
            "priority": r.priority.unwrap_or_default(),
            "count": r.count.unwrap_or(0)
        })).collect::<Vec<_>>(),
        "cases_by_status": cases_by_status.into_iter().map(|r| json!({
            "status": r.status.unwrap_or_default(),
            "count": r.count.unwrap_or(0)
        })).collect::<Vec<_>>()
    })))
}

pub async fn get_document_analytics(
    State(db): State<SqlitePool>,
    Query(params): Query<HashMap<String, String>>,
) -> Result<Json<Value>, StatusCode> {
    let days = params
        .get("days")
        .and_then(|d| d.parse().ok())
        .unwrap_or(30);

    // Documents created in the last N days
    let documents_by_day = sqlx::query!(
        r#"
        SELECT DATE(created_at) as date, COUNT(*) as count
        FROM documents 
        WHERE created_at >= datetime('now', '-' || ? || ' days')
        GROUP BY DATE(created_at)
        ORDER BY date
        "#,
        days
    )
    .fetch_all(&db)
    .await
    .unwrap_or_default();

    // Documents by status
    let documents_by_status = sqlx::query!(
        r#"
        SELECT status, COUNT(*) as count
        FROM documents
        GROUP BY status
        "#
    )
    .fetch_all(&db)
    .await
    .unwrap_or_default();

    // Document file types
    let documents_by_type = sqlx::query!(
        r#"
        SELECT 
            CASE 
                WHEN mime_type LIKE 'image/%' THEN 'Image'
                WHEN mime_type LIKE 'application/pdf' THEN 'PDF'
                WHEN mime_type LIKE 'application/vnd.ms-%' OR mime_type LIKE 'application/vnd.openxmlformats%' THEN 'Office'
                WHEN mime_type LIKE 'text/%' THEN 'Text'
                ELSE 'Other'
            END as file_type,
            COUNT(*) as count
        FROM documents
        WHERE mime_type IS NOT NULL
        GROUP BY file_type
        "#
    )
    .fetch_all(&db)
    .await
    .unwrap_or_default();

    Ok(Json(json!({
        "period_days": days,
        "documents_by_day": documents_by_day.into_iter().map(|r| json!({
            "date": r.date.unwrap_or_default(),
            "count": r.count.unwrap_or(0)
        })).collect::<Vec<_>>(),
        "documents_by_status": documents_by_status.into_iter().map(|r| json!({
            "status": r.status.unwrap_or_default(),
            "count": r.count.unwrap_or(0)
        })).collect::<Vec<_>>(),
        "documents_by_type": documents_by_type.into_iter().map(|r| json!({
            "file_type": r.file_type.unwrap_or_default(),
            "count": r.count.unwrap_or(0)
        })).collect::<Vec<_>>()
    })))
}

pub async fn get_user_activity_report(
    State(db): State<SqlitePool>,
    Query(params): Query<HashMap<String, String>>,
) -> Result<Json<Value>, StatusCode> {
    let days = params
        .get("days")
        .and_then(|d| d.parse().ok())
        .unwrap_or(7);

    // Most active users
    let active_users = sqlx::query!(
        r#"
        SELECT u.id, u.username, u.full_name, COUNT(a.id) as activity_count
        FROM users u
        LEFT JOIN activities a ON u.id = a.user_id 
            AND a.created_at >= datetime('now', '-' || ? || ' days')
        GROUP BY u.id, u.username, u.full_name
        ORDER BY activity_count DESC
        LIMIT 10
        "#,
        days
    )
    .fetch_all(&db)
    .await
    .unwrap_or_default();

    // Activity by action type
    let activities_by_action = sqlx::query!(
        r#"
        SELECT action, COUNT(*) as count
        FROM activities
        WHERE created_at >= datetime('now', '-' || ? || ' days')
        GROUP BY action
        ORDER BY count DESC
        "#,
        days
    )
    .fetch_all(&db)
    .await
    .unwrap_or_default();

    Ok(Json(json!({
        "period_days": days,
        "most_active_users": active_users.into_iter().map(|r| json!({
            "user_id": r.id.unwrap_or_default(),
            "username": r.username.unwrap_or_default(),
            "full_name": r.full_name,
            "activity_count": r.activity_count.unwrap_or(0)
        })).collect::<Vec<_>>(),
        "activities_by_action": activities_by_action.into_iter().map(|r| json!({
            "action": r.action.unwrap_or_default(),
            "count": r.count.unwrap_or(0)
        })).collect::<Vec<_>>()
    })))
}

pub async fn get_system_health_metrics(
    State(db): State<SqlitePool>,
) -> Result<Json<Value>, StatusCode> {
    // Database statistics
    let db_stats = sqlx::query!(
        r#"
        SELECT 
            (SELECT COUNT(*) FROM users) as user_count,
            (SELECT COUNT(*) FROM documents) as document_count,
            (SELECT COUNT(*) FROM cases) as case_count,
            (SELECT COUNT(*) FROM teams) as team_count,
            (SELECT COUNT(*) FROM notifications WHERE read_at IS NULL) as unread_notifications,
            (SELECT COUNT(*) FROM activities WHERE created_at >= datetime('now', '-1 day')) as recent_activities
        "#
    )
    .fetch_one(&db)
    .await;

    match db_stats {
        Ok(stats) => Ok(Json(json!({
            "database_health": "healthy",
            "metrics": {
                "total_users": stats.user_count.unwrap_or(0),
                "total_documents": stats.document_count.unwrap_or(0),
                "total_cases": stats.case_count.unwrap_or(0),
                "total_teams": stats.team_count.unwrap_or(0),
                "unread_notifications": stats.unread_notifications.unwrap_or(0),
                "recent_activities_24h": stats.recent_activities.unwrap_or(0)
            },
            "timestamp": chrono::Utc::now().to_rfc3339()
        }))),
        Err(err) => {
            error!("Failed to get system health metrics: {:?}", err);
            Err(StatusCode::INTERNAL_SERVER_ERROR)
        }
    }
}

// Helper function to calculate average case resolution time
async fn calculate_avg_resolution_time(db: &SqlitePool) -> Result<f64, sqlx::Error> {
    let result = sqlx::query!(
        r#"
        SELECT 
            AVG(julianday(updated_at) - julianday(created_at)) * 24 as avg_hours
        FROM cases 
        WHERE status IN ('closed', 'resolved')
        "#
    )
    .fetch_one(db)
    .await?;
    
    Ok(result.avg_hours.unwrap_or(0.0))
}
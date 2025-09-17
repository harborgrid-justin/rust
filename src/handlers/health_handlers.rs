use axum::{extract::State, http::StatusCode, response::Json};
use serde_json::{json, Value};
use sqlx::SqlitePool;

pub async fn health_check() -> Result<Json<Value>, StatusCode> {
    Ok(Json(json!({
        "status": "healthy",
        "service": "Enterprise Document and Case Management Platform",
        "version": env!("CARGO_PKG_VERSION"),
        "timestamp": chrono::Utc::now()
    })))
}

pub async fn readiness_check(State(db): State<SqlitePool>) -> Result<Json<Value>, StatusCode> {
    // Check database connectivity
    match sqlx::query("SELECT 1").execute(&db).await {
        Ok(_) => Ok(Json(json!({
            "status": "ready",
            "database": "connected",
            "timestamp": chrono::Utc::now()
        }))),
        Err(_) => Err(StatusCode::SERVICE_UNAVAILABLE),
    }
}
use axum::{
    extract::{Path, Query, State},
    http::StatusCode,
    response::Json,
};
use chrono;
use serde_json::{json, Value};
use sqlx::SqlitePool;
use std::collections::HashMap;
use tracing::{error, info};
use validator::Validate;

use crate::models::{CreateCaseRequest, UpdateCaseRequest, CaseResponse, DocumentResponse, CasePriority};
use crate::services::case_service;

pub async fn list_cases(
    State(db): State<SqlitePool>,
    Query(params): Query<HashMap<String, String>>,
) -> Result<Json<Value>, StatusCode> {
    let status = params.get("status");
    let assigned_to = params.get("assigned_to");
    let limit = params
        .get("limit")
        .and_then(|l| l.parse::<i32>().ok())
        .unwrap_or(50);
    let offset = params
        .get("offset")
        .and_then(|o| o.parse::<i32>().ok())
        .unwrap_or(0);

    match case_service::list_cases(&db, status, assigned_to, limit, offset).await {
        Ok(cases) => {
            let responses: Vec<CaseResponse> = cases
                .into_iter()
                .map(CaseResponse::from)
                .collect();
            
            Ok(Json(json!({
                "cases": responses,
                "count": responses.len()
            })))
        }
        Err(err) => {
            error!("Failed to list cases: {:?}", err);
            Err(StatusCode::INTERNAL_SERVER_ERROR)
        }
    }
}

pub async fn create_case(
    State(db): State<SqlitePool>,
    Json(payload): Json<CreateCaseRequest>,
) -> Result<Json<CaseResponse>, StatusCode> {
    // Basic validation using validator derive
    if let Err(err) = payload.validate() {
        error!("Case validation failed: {:?}", err);
        return Err(StatusCode::BAD_REQUEST);
    }

    // Business logic validation
    if let Err(validation_error) = validate_case_business_rules(&db, &payload).await {
        error!("Case business rule validation failed: {}", validation_error);
        return Err(StatusCode::BAD_REQUEST);
    }

    // TODO: Extract user ID from JWT token
    let created_by = "system".to_string(); // Temporary placeholder

    match case_service::create_case(&db, payload, created_by).await {
        Ok(case) => {
            info!("Case created successfully: {} ({})", case.title, case.id);
            Ok(Json(CaseResponse::from(case)))
        },
        Err(err) => {
            error!("Failed to create case: {:?}", err);
            Err(StatusCode::INTERNAL_SERVER_ERROR)
        }
    }
}

pub async fn get_case(
    State(db): State<SqlitePool>,
    Path(id): Path<String>,
) -> Result<Json<CaseResponse>, StatusCode> {
    match case_service::get_case(&db, &id).await {
        Ok(Some(case)) => Ok(Json(CaseResponse::from(case))),
        Ok(None) => Err(StatusCode::NOT_FOUND),
        Err(err) => {
            error!("Failed to get case: {:?}", err);
            Err(StatusCode::INTERNAL_SERVER_ERROR)
        }
    }
}

pub async fn update_case(
    State(db): State<SqlitePool>,
    Path(id): Path<String>,
    Json(payload): Json<UpdateCaseRequest>,
) -> Result<Json<CaseResponse>, StatusCode> {
    if let Err(err) = payload.validate() {
        error!("Case update validation failed: {:?}", err);
        return Err(StatusCode::BAD_REQUEST);
    }

    // TODO: Extract user ID from JWT token for audit trail
    let updated_by = "system".to_string(); // Temporary placeholder

    match case_service::update_case(&db, &id, payload, updated_by).await {
        Ok(Some(case)) => Ok(Json(CaseResponse::from(case))),
        Ok(None) => Err(StatusCode::NOT_FOUND),
        Err(err) => {
            error!("Failed to update case: {:?}", err);
            Err(StatusCode::INTERNAL_SERVER_ERROR)
        }
    }
}

pub async fn get_case_documents(
    State(db): State<SqlitePool>,
    Path(case_id): Path<String>,
) -> Result<Json<Value>, StatusCode> {
    match case_service::get_case_documents(&db, &case_id).await {
        Ok(documents) => {
            let responses: Vec<DocumentResponse> = documents
                .into_iter()
                .map(DocumentResponse::from)
                .collect();
            
            Ok(Json(json!({
                "case_id": case_id,
                "documents": responses,
                "count": responses.len()
            })))
        }
        Err(err) => {
            error!("Failed to get case documents: {:?}", err);
            Err(StatusCode::INTERNAL_SERVER_ERROR)
        }
    }
}

pub async fn add_document_to_case(
    State(db): State<SqlitePool>,
    Path((case_id, doc_id)): Path<(String, String)>,
) -> Result<Json<Value>, StatusCode> {
    // TODO: Extract user ID from JWT token
    let added_by = "system".to_string(); // Temporary placeholder

    match case_service::add_document_to_case(&db, &case_id, &doc_id, added_by).await {
        Ok(_) => Ok(Json(json!({
            "message": "Document added to case successfully",
            "case_id": case_id,
            "document_id": doc_id
        }))),
        Err(err) => {
            error!("Failed to add document to case: {:?}", err);
            Err(StatusCode::INTERNAL_SERVER_ERROR)
        }
    }
}

// Business logic validation for case creation
async fn validate_case_business_rules(
    db: &SqlitePool,
    request: &CreateCaseRequest,
) -> Result<(), String> {
    // 1. Duplicate detection using title similarity
    if let Err(_) = check_duplicate_case_title(db, &request.title).await {
        return Err("Similar case title already exists".to_string());
    }

    // 2. Business rule: Priority vs urgency matrix validation
    if let Some(priority) = &request.priority {
        if !validate_priority_assignment(priority, &request.description) {
            return Err("Priority assignment doesn't match case description indicators".to_string());
        }
    }

    // 3. Validate assigned user exists (if provided)
    if let Some(assigned_to) = &request.assigned_to {
        if !user_exists(db, assigned_to).await {
            return Err("Assigned user does not exist".to_string());
        }
    }

    // 4. Due date validation - should be in the future
    if let Some(due_date_str) = &request.due_date {
        if let Ok(due_date) = chrono::DateTime::parse_from_rfc3339(due_date_str) {
            if due_date < chrono::Utc::now() {
                return Err("Due date cannot be in the past".to_string());
            }
        } else {
            return Err("Invalid due date format, expected RFC3339".to_string());
        }
    }

    Ok(())
}

// Check for similar case titles using basic string similarity
async fn check_duplicate_case_title(db: &SqlitePool, title: &str) -> Result<(), sqlx::Error> {
    let similar_cases = sqlx::query!(
        r#"
        SELECT COUNT(*) as count
        FROM cases 
        WHERE title = ?1 OR title LIKE ?2
        "#,
        title,
        format!("%{}%", title)
    )
    .fetch_one(db)
    .await?;

    if similar_cases.count.unwrap_or(0) > 0 {
        return Err(sqlx::Error::RowNotFound);
    }

    Ok(())
}

// Validate priority assignment based on description keywords
fn validate_priority_assignment(priority: &CasePriority, description: &Option<String>) -> bool {
    if let Some(desc) = description {
        let desc_lower = desc.to_lowercase();
        let urgent_keywords = ["urgent", "emergency", "critical", "asap", "immediate"];
        let high_keywords = ["important", "high", "priority", "escalate"];
        let low_keywords = ["minor", "low", "enhancement", "future"];

        match priority {
            CasePriority::High | CasePriority::Critical => {
                // High/Critical priority should have urgent or high priority keywords
                urgent_keywords.iter().any(|&keyword| desc_lower.contains(keyword)) ||
                high_keywords.iter().any(|&keyword| desc_lower.contains(keyword))
            },
            CasePriority::Low => {
                // Low priority should not have urgent keywords
                !urgent_keywords.iter().any(|&keyword| desc_lower.contains(keyword))
            },
            CasePriority::Medium => {
                // Medium priority is flexible, but shouldn't have urgent keywords for low
                true // Medium is always acceptable
            },
        }
    } else {
        true // No description means we can't validate, so allow it
    }
}

// Check if user exists in the database
async fn user_exists(db: &SqlitePool, user_id: &str) -> bool {
    sqlx::query!("SELECT id FROM users WHERE id = ?1", user_id)
        .fetch_optional(db)
        .await
        .map(|result| result.is_some())
        .unwrap_or(false)
}
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

use crate::models::{CreateCaseRequest, UpdateCaseRequest, CaseResponse, DocumentResponse};
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
    if let Err(err) = payload.validate() {
        error!("Case validation failed: {:?}", err);
        return Err(StatusCode::BAD_REQUEST);
    }

    // TODO: Extract user ID from JWT token
    let created_by = "system".to_string(); // Temporary placeholder

    match case_service::create_case(&db, payload, created_by).await {
        Ok(case) => Ok(Json(CaseResponse::from(case))),
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